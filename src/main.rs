use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::io;
use std::env;

// ----------------------------------------------------
// ## 1. STRUCTIT JA ENUM (TÄYTYY OLLA KAIKKEA MUUTA ENNEN)
// ----------------------------------------------------

// POHJASTRUCTIT
#[derive(Debug, Deserialize)]
struct Source {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Article {
    title: String,
    author: Option<String>,
    source: Source,
    url: String,
}

// VIRHEEN KÄSITTELY STRUCTIT
#[derive(Debug, Deserialize)]
struct ApiError {
    status: String,
    code: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct NewsResponse {
    status: String,
    articles: Vec<Article>,
}

// ENUM: Vastaus on joko onnistunut (Ok) tai virhe (Err)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse {
    Ok(NewsResponse),
    Err(ApiError),
}

// ----------------------------------------------------
// ## 2. APUFUNKTIOT
// ----------------------------------------------------

// Tulostaa artikkelit siististi käyttäjälle
fn print_articles(articles: Vec<Article>) {
    if articles.is_empty() {
        println!("Ei tuloksia.");
        return;
    }

    println!("\n--- Tulokset ---");
    for article in articles {
        println!("> {} ({})", article.title, article.source.name);
        println!("  Lue lisää: {}\n", article.url);
    }
}

// ----------------------------------------------------
// ## 3. HAKUFUNKTIOT
// ----------------------------------------------------

// Hakee tuoreimmat otsikot (top-headlines)
async fn get_top_headlines(api_key: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    println!("Haetaan uusimpia otsikoita...");

    let url = format!(
        "https://newsapi.org/v2/top-headlines?country=us&apiKey={}",
        api_key
    );

    let client = reqwest::Client::new();
    let api_response: ApiResponse = client
        .get(&url)
        .header(USER_AGENT, "rust-news-fetcher-app")
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;

    match api_response {
        ApiResponse::Ok(response) => Ok(response.articles),
        ApiResponse::Err(e) => Err(format!("API Virhe: {} ({})", e.message, e.code).into()),
    }
}

// Hakee uutisia vapaalla hakusanalla (everything)
async fn get_news_with_query(query: &str, api_key: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    println!("Haetaan uutisia hakusanalla: {}...", query);

    let url = format!(
        "https://newsapi.org/v2/everything?q={}&sortBy=popularity&apiKey={}",
        query, api_key
    );

    let client = reqwest::Client::new();
    let api_response: ApiResponse = client
        .get(&url)
        .header(USER_AGENT, "rust-news-fetcher-app")
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;

    match api_response {
        ApiResponse::Ok(response) => Ok(response.articles),
        ApiResponse::Err(e) => Err(format!("API Virhe: {} ({})", e.message, e.code).into()),
    }
}

// ----------------------------------------------------
// ## 4. PÄÄOHJELMA (main)
// ----------------------------------------------------

#[tokio::main]
async fn main() {
    println!("Uutishaku-ohjelma käynnistyy...");

    // 1. Ladataan ympäristömuuttujat .env tiedostosta (vaatii 'dotenv' kirjaston)
    dotenv::dotenv().ok();
    
    // 2. Luetaan API-avain. Jos avainta ei löydy, ohjelma lopettaa.
    let api_key = match env::var("NEWS_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("VIRHE: API-avainta 'NEWS_API_KEY' ei löytynyt ympäristömuuttujista tai .env-tiedostosta.");
            eprintln!("Varmista, että olet luonut .env-tiedoston ja lisännyt siihen: NEWS_API_KEY=OmaAvain");
            std::process::exit(1);
        }
    };
    let api_key_ref = api_key.as_str();

    loop {
        println!("\nMitä haluat tehdä?");
        println!("  1. Näytä tuoreimmat otsikot (vaatii täyden API-avaimen)");
        println!("  2. Hae uutisia hakusanalla");
        println!("  q. Poistu");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Valinnan lukeminen epäonnistui");

        match choice.trim() {
            "1" => {
                match get_top_headlines(api_key_ref).await {
                    Ok(articles) => print_articles(articles),
                    Err(e) => println!("Virhe uutisia haettaessa: {}", e),
                }
            }
            "2" => {
                println!("Kirjoita hakusana:");
                let mut search_term = String::new();
                io::stdin()
                    .read_line(&mut search_term)
                    .expect("Hakusanan lukeminen epäonnistui");

                match get_news_with_query(search_term.trim(), api_key_ref).await {
                    Ok(articles) => print_articles(articles),
                    Err(e) => println!("Virhe uutisia haettaessa: {}", e),
                }
            }
            "q" | "Q" => {
                println!("Näkemiin!");
                break;
            }
            _ => {
                println!("Virheellinen valinta, yritä uudelleen.");
            }
        }
    }
}