# Smoke Exporter (MTI 882)

Le Smoke Exporter est un outil compact conçu pour convertir des données de fumée au format OpenVDB en fichiers binaires. Cet utilitaire est écrit en Rust et nécessite un compilateur pour fonctionner correctement. L'utilisation de l'outil se fait en ligne de commande, offrant une flexibilité dans le processus de conversion.

## Utilisation
```bash
Usage: smoke_conversion.exe [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>                fichier de test
  -g, --grid <GRID>                  grille à charger [default: density_noise]
  -o, --output <OUTPUT>              fichier de sortie [default: out]
  -u, --use-metadata <USE_METADATA>  Use metadata to get the grid size
  -h, --help                         Print help
  -V, --version                      Print version
```

Les paramètres sont:
- `-i, --input <CHEMIN_VERS_LE_FICHIER>`: Spécifie le chemin vers le fichier OpenVDB à convertir.
- `-g, --grid <NOM_DE_LA_GRILLE>`: Sélectionne la grille à extraire du fichier OpenVDB. La liste des grilles disponibles est affichée en début de commande. Par défaut, la grille utilisée est "density_noise".
- `-o, --output <NOM_DU_FICHIER_SORTIE>`: Spécifie le nom du fichier de sortie généré. Par défaut, le fichier de sortie est nommé "out".
- `-u, --use-metadata <UTILISER_METADATA>`: Utilise la métadonnée pour obtenir la taille de la grille. Si reseignée, la résolution de la grille est déterminée à partir de la métadonnée. Généralement disponible via `file_base_resolution`. Sinon, une heuristique basée sur la construction d'une boîte englobante est utilisée.

## Format binaire
Le fichier généré est au format binaire et comprend les informations suivantes:
```
size_x
size_y
size_z
d0
...
dN
```
Les trois premiers paramètres (size_x, size_y, size_z) en int32 définissent la taille du volume. Ensuite, les densités sont définies dans l'ordre `x + y * size_x + z * size_x * size_y`, où `(x, y, z)` sont les coordonnées du voxel.


## Pseudo-code C++
Voici le pseudo-code C++ pour lire ce fichier (généré par ChatGPT)
```cpp
#include <iostream>
#include <fstream>
#include <vector>

struct VoxelData {
    int size_x;
    int size_y;
    int size_z;
    std::vector<double> densities;
};

bool readBinaryFile(const std::string& filename, VoxelData& voxelData) {
    std::ifstream file(filename, std::ios::binary);
    
    if (!file.is_open()) {
        std::cerr << "Erreur : Impossible d'ouvrir le fichier " << filename << std::endl;
        return false;
    }

    file.read(reinterpret_cast<char*>(&voxelData.size_x), sizeof(int));
    file.read(reinterpret_cast<char*>(&voxelData.size_y), sizeof(int));
    file.read(reinterpret_cast<char*>(&voxelData.size_z), sizeof(int));

    int totalVoxels = voxelData.size_x * voxelData.size_y * voxelData.size_z;
    voxelData.densities.resize(totalVoxels);

    file.read(reinterpret_cast<char*>(voxelData.densities.data()), totalVoxels * sizeof(double));

    file.close();

    return true;
}
```
