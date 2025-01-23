# Analyseur de Texte Web - Rust API

Un outil d'analyse textuelle puissant développé en Rust qui permet d'analyser le contenu de plusieurs pages web simultanément. Il extrait des statistiques détaillées et identifie les expressions les plus pertinentes.

## 🚀 Fonctionnalités

### Analyse de Texte
- 📊 Calcul des fréquences de mots et n-grammes
- 📏 Analyse de la longueur moyenne des mots
- 🔍 Identification des expressions clés
- 📈 Statistiques détaillées par document
- 🌐 Support multi-URL
- 🔄 Traitement parallèle des requêtes

## 📋 Prérequis

- Rust (version 1.70 ou supérieure)
- Cargo
- Fichier de mots à filtrer (stop_words_french.txt)

## 🛠️ Installation

1. Clonez le dépôt :
```bash
git clone https://github.com/yu-Celik/rust-text-analyzer.git
cd rust-text-analyzer
```

2. Installez les dépendances :
```bash
cargo build
```

## 💻 Utilisation

### Démarrage du serveur

```bash
cargo run
```

Le serveur démarre sur `http://localhost:8080` par défaut.

### Exemple d'appel API

```typescript
const response = await fetch('http://localhost:8080/api/analyze', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
    },
    body: JSON.stringify({
        urls: [
            "https://example.com/page1",
            "https://example.com/page2"
        ],
        ngrams_to_analyze: [1, 2, 3]
    })
});
```

### Format de Réponse

```json
{
    "frequencies": [
        {
            "expression": "exemple expression",
            "gram_type": "bigramme",
            "average_occurrences": 12.5,
            "average_percentage": 3.45,
            "doc_count_percentage": 75.0,
            "sources": ["url1", "url2"]
        }
    ],
    "document_stats": [
        {
            "url": "https://example.com",
            "total_retained": 150,
            "total_unique": 80,
            "word_count": 200,
            "average_word_length": 5.7
        }
    ],
    "url_statuses": [
        {
            "url": "https://example.com",
            "status": "ok",
            "error": null
        }
    ]
}
```

## 🧪 Tests

Exécutez les tests unitaires :
```bash
cargo test
```

## 📚 Documentation

Générez la documentation :
```bash
cargo doc --open
```

## 📝 Licence

Ce projet est sous licence MIT - voir le fichier [LICENSE](LICENSE) pour plus de détails.

## 🙏 Remerciements

- Équipe Rust pour le langage et l'écosystème
- Contributeurs des bibliothèques utilisées
- Communauté open source

Lien du projet : [https://github.com/yu-Celik/rust-text-analyzer](https://github.com/yu-Celik/rust-text-analyzer)