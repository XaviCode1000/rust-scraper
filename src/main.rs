mod config;
mod markdown;
mod scraper;

use std::path::Path;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inicializar el sistema de logging
    config::init_logging();
    info!("🚀 Iniciando Brave RAG Scraper v2");

    let target_url = "https://docs.rs/spider/latest/spider/";
    let output_dir = Path::new("rag_dataset");

    // 2. Validar URL
    validate_url(target_url)?;
    info!("✅ URL validada: {}", target_url);

    // 3. Configurar el entorno de Brave
    config::setup_brave_env()?;

    // 4. Ejecutar el crawler
    info!("📡 Iniciando scraping...");
    let pages = scraper::crawl_target(target_url).await;

    if pages.is_empty() {
        return Err("No se obtuvieron páginas del sitio".into());
    }

    info!("✅ Scraping completado: {} páginas obtenidas", pages.len());

    // 5. Procesar y guardar como Markdown
    info!("📝 Procesando contenido a Markdown...");
    markdown::process_and_save(&pages, output_dir)?;

    info!("🎉 Pipeline RAG completado exitosamente");
    Ok(())
}

/// Valida que una URL sea bien formada
fn validate_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("URL debe comenzar con http:// o https://: {}", url).into());
    }
    Ok(())
}
