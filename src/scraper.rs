use spider::page::Page;
use spider::website::Website;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuración por defecto para el crawler
const DEFAULT_DELAY_MS: u64 = 250;
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Realiza el scraping de un sitio web usando Brave como navegador
///
/// # Argumentos
///
/// * `url` - URL del sitio a scrapear
///
/// # Retorna
///
/// Un vector de páginas renderizadas por Brave. Si ocurre un error durante
/// el crawling, devuelve un vector vacío y registra una advertencia.
///
/// # Ejemplo
///
/// ```no_run
/// let pages = crawl_target("https://example.com").await;
/// println!("Se obtuvieron {} páginas", pages.len());
/// ```
pub async fn crawl_target(url: &str) -> Vec<Page> {
    if url.is_empty() {
        warn!("URL vacía proporcionada");
        return Vec::new();
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        warn!("URL debe comenzar con http:// o https://: {}", url);
        return Vec::new();
    }

    info!("🦁 Iniciando scraping en: {}", url);

    let mut website = Website::new(url);

    // Configurar el crawler con los parámetros óptimos
    website.configuration.respect_robots_txt = true;
    website.configuration.delay = DEFAULT_DELAY_MS;
    website.configuration.request_timeout =
        Some(Box::new(Duration::from_millis(DEFAULT_TIMEOUT_MS)));

    debug!("Configuración del crawler establecida:");
    debug!(
        "  - Respetar robots.txt: {}",
        website.configuration.respect_robots_txt
    );
    debug!(
        "  - Delay entre requests: {}ms",
        website.configuration.delay
    );
    debug!("  - Timeout: {}ms", DEFAULT_TIMEOUT_MS);

    // Ejecutar el crawler
    // El feature "chrome" en Cargo.toml activa el modo Headless CDP automáticamente
    website.crawl().await;

    // Obtener las páginas renderizadas
    let pages = website.get_pages().cloned().unwrap_or_default();

    if pages.is_empty() {
        warn!("⚠️  No se obtuvieron páginas del sitio: {}", url);
    } else {
        info!("✅ Se obtuvieron {} páginas", pages.len());
    }

    pages
}
