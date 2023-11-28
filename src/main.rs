use std::{fs::File, io::BufReader};
use cgmath::Point3;
use clap::Parser;
use log::{info, error, warn};
use vdb_rs::{VdbReader, VdbLevel};

use byteorder::WriteBytesExt; // This trait adds methods to writeable types
use byteorder::LittleEndian;


#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Point3<f64>,
    pub max: Point3<f64>,
}
impl Default for AABB {
    fn default() -> Self {
        let min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        Self { min, max }
    }
}
impl AABB {
    pub fn extend(&mut self, v: Point3<f64>) {
        self.min.x = self.min.x.min(v.x);
        self.min.y = self.min.y.min(v.y);
        self.min.z = self.min.z.min(v.z);

        self.max.x = self.max.x.max(v.x);
        self.max.y = self.max.y.max(v.y);
        self.max.z = self.max.z.max(v.z);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// fichier de test
    #[arg(short, long)]
    input: String,

    /// grille à charger
    #[arg(short, long, default_value = "density_noise")]
    grid: String,

    /// fichier de sortie
    #[arg(short, long, default_value = "out")]
    output: String,

    /// Use metadata to get the grid size
    #[arg(short, long)]
    use_metadata: Option<String>,
}

fn main() -> std::io::Result<()> {
     // Lecture de la ligne de commande
     let args = Args::parse();
     pretty_env_logger::formatted_builder()
         .filter_level(log::LevelFilter::Info)
         .init();

    info!("Lecture du fichier {}", args.input);
    
    let f = File::open(args.input).unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
    let grid_names = vdb_reader.available_grids();
    info!("Available grids: {:?}", grid_names);

    // Montrer les metadata
    vdb_reader.grid_descriptors.iter().for_each(|g| {
        info!("Grid: {}", g.0);
        for (k, v) in g.1.meta_data.0.iter() {
            info!("  - {}: {:?}", k, v);
        }
    });

    // Normalement on va charger "density"
    let grid_to_load = args.grid;
    if grid_names.iter().find(|&s| s == &grid_to_load).is_none() {
        error!("Grid {} not found. See availabe grids", grid_to_load);
        return Ok(());
    }

    // On charge la "grille ici" -- on ne peut pas utiliser cette grille directement
    let grid = vdb_reader.read_grid::<f32>(&grid_to_load).unwrap();

    let mut nb_voxels = 0;
    let mut aabb_smoke = AABB::default();
    for (pos, _, level) in grid.iter() {
        if level == VdbLevel::Voxel {
            aabb_smoke.extend(Point3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64));
            nb_voxels += 1;
        } else {
            warn!("Other level than voxel are ignored");
        }
    }
    
    // Compute size nearest
    let (size_x, size_y, size_z) = if let Some(key) = args.use_metadata {
        let res = grid.descriptor.meta_data.0.get(key.as_str());
        if res.is_none() {
            error!("Key {} not found in metadata. Please check the name", key);
            return Ok(());
        }
        let res = res.unwrap();
        match res {
            vdb_rs::MetadataValue::Vec3i(v) => (v[0] as usize, v[1] as usize, v[2] as usize),
            _ => {
                error!("Key {} is not a Vec3i. Please check the name", key);
                return Ok(());
            }
        }
    } else { 
        (aabb_smoke.max.x as usize + 1, aabb_smoke.max.y as usize + 1, aabb_smoke.max.z as usize + 1)
    };
    info!("AABB (Smoke): {:?}", aabb_smoke);
    info!("Size: {}x{}x{}", size_x, size_y, size_z);
    info!("Filled density: {} / {}", nb_voxels, size_x * size_y * size_z);
    
    // Compute the density
    let mut density = vec![0.0; (size_x * size_y * size_z) as usize];
    let mut max_density = 0.0_f64;
    for (pos, voxel, level) in grid.iter() {
        if level == VdbLevel::Voxel {
            let x = pos[0]  as usize;
            let y = pos[1] as usize;
            let z = pos[2] as usize;
            let idx = x + y * size_x as usize + z * size_x as usize * size_y as usize;
            // Density save
            max_density = max_density.max(voxel as f64);
            density[idx] = voxel as f64;
        }
    }

    // Save density
    info!("Max density: {}", max_density);
    info!("Saving density to {}.density", args.output);
    let mut density_file = File::create(format!("{}.density", args.output)).unwrap();
    density_file.write_i32::<LittleEndian>(size_x as i32)?;
    density_file.write_i32::<LittleEndian>(size_y as i32)?;
    density_file.write_i32::<LittleEndian>(size_z as i32)?;
    for v in density {
        density_file.write_f64::<LittleEndian>(v)?;
    }

    info!("Done");

    Ok(())
}
