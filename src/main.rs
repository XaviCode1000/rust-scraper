//! Rust Scraper - Modern web scraper for RAG datasets
//!
//! Extracts clean, structured content from web pages using readability algorithm.

use anyhow::Context;
use rust_scraper::{config, scraper, validate_and_parse_url, Args, Parser, ScraperConfig};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Parsear argumentos CLI - Si no hay URL, error inmediato y claro
    let args = Args::parse();

    // 2. Inicializar logging con nivel configurable
    let log_level = match args.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    config::init_logging(log_level);

    info!("🚀 Rust Scraper v0.2.0 - Modern Stack");
    info!("📌 Target: {}", args.url);
    info!("📁 Output: {}", args.output.display());

    // 3. Validar URL - parsing con la crate url
    let parsed_url = validate_and_parse_url(&args.url).context("Invalid URL provided")?;

    info!("✅ URL validada: {}", parsed_url);

    // 4. Crear cliente HTTP configurado
    let client = scraper::create_http_client()?;

    // 5. Configurar scraping con opciones de descarga
    let config = ScraperConfig {
        download_images: args.download_images,
        download_documents: args.download_documents,
        output_dir: args.output.clone(),
        max_file_size: Some(50 * 1024 * 1024), // 50MB default
    };

    if config.download_images {
        info!("🖼️  Image download: ENABLED");
    }
    if config.download_documents {
        info!("📄 Document download: ENABLED");
    }

    // 6. Ejecutar scraping
    info!("📡 Iniciando scraping...");

    let results = scraper::scrape_with_config(&client, &parsed_url, &config)
        .await
        .context("Scraping failed")?;

    if results.is_empty() {
        warn!("⚠️  No se obtuvo contenido de la página");
        return Ok(());
    }

    info!(
        "✅ Scraping completado: {} elementos extraídos",
        results.len()
    );

    // 7. Guardar resultados
    info!("💾 Guardando resultados...");
    scraper::save_results(&results, &args.output, &args.format)?;

    // Resumen de assets descargados
    let total_assets: usize = results.iter().map(|r| r.assets.len()).sum();
    if total_assets > 0 {
        info!(
            "📦 Total assets descargados: {} (imágenes y documentos)",
            total_assets
        );
    }

    info!("🎉 Pipeline completado exitosamente!");
    info!("📊 Archivos generados: {}", args.output.display());

    Ok(())
}
