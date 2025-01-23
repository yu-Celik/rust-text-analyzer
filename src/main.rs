mod tests;
mod text_analyzer;
mod web_analyzer;
mod api;
use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = "0.0.0.0";
    
    println!("Serveur démarré sur http://{}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(api::analyze_urls)
    })
    .bind((host, port.parse::<u16>().unwrap()))?
    .run()
    .await
}


// "https://www.onisep.fr/ressources/univers-metier/metiers/professeur-professeure-de-college-et-de-lycee",
// "https://www.larousse.fr/dictionnaires/francais/professeur/64155",
// "https://www.devenirenseignant.gouv.fr/",
// "https://fr.wikipedia.org/wiki/Professeur_(titre)",
// "https://dictionnaire.lerobert.com/definition/professeur",
// "https://www.hellowork.com/fr-fr/metiers/professeur.html",
// "https://fr.wikipedia.org/wiki/Enseignant",
// "https://fr.wikipedia.org/wiki/Professeur",
// "https://www.dictionnaire-academie.fr/article/A9P4464",
// "https://www.letudiant.fr/etudes/devenir-professeur-mode-demploi.html"