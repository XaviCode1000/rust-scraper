use spider::page::Page;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errores relacionados con el procesamiento de markdown
#[derive(Error, Debug)]
pub enum MarkdownError {
    #[error("Error de I/O: {0}")]
    IoError(#[from] std::io::Error),

    #[error("No hay páginas para procesar")]
    NoPagesProvided,
}

/// Procesa las páginas HTML y las guarda como archivos Markdown
///
/// # Argumentos
///
/// * `pages` - Vector de páginas obtenidas del crawler
/// * `output_dir` - Directorio donde guardar los archivos Markdown
///
/// # Retorna
///
/// `Ok(())` si todas las conversiones se completaron exitosamente,
/// o un `MarkdownError` si ocurrió algún problema.
///
/// # Errores
///
/// - Si no hay páginas para procesar
/// - Si no se puede crear el directorio de salida
/// - Si no se pueden escribir los archivos de salida
pub fn process_and_save(pages: &[Page], output_dir: &Path) -> Result<(), MarkdownError> {
    if pages.is_empty() {
        warn!("⚠️  No hay páginas para procesar");
        return Err(MarkdownError::NoPagesProvided);
    }

    let output_dir_str = output_dir.to_string_lossy().to_string();

    // Crear directorio de salida
    fs::create_dir_all(output_dir)?;
    info!("📁 Directorio de salida creado: {}", output_dir_str);

    let total_pages = pages.len();
    let mut successful = 0;
    let mut failed = 0;

    for (i, page) in pages.iter().enumerate() {
        match process_single_page(page, i, output_dir) {
            Ok(_) => {
                successful += 1;
                debug!("Página {}/{} procesada", i + 1, total_pages);
            }
            Err(e) => {
                warn!("Error al procesar página {}: {}", i, e);
                failed += 1;
            }
        }
    }

    info!(
        "✅ Conversión completada: {} exitosas, {} fallidas",
        successful, failed
    );

    Ok(())
}

/// Procesa una única página y la guarda como archivo Markdown
fn process_single_page(page: &Page, index: usize, output_dir: &Path) -> Result<(), MarkdownError> {
    let html_content = page.get_html();

    // Validar que hay contenido HTML
    if html_content.is_empty() {
        warn!("⚠️  Página {} no tiene contenido HTML", index);
        return Ok(()); // No es un error crítico
    }

    // Convertir HTML a Markdown
    let markdown = html_to_markdown(&html_content);

    // Generar nombre de archivo
    let file_name = format!("doc_{:03}.md", index);
    let file_path = output_dir.join(&file_name);

    // Guardar archivo
    fs::write(&file_path, markdown)?;
    info!("✅ Documento guardado: {}", file_path.display());

    Ok(())
}

/// Convierte HTML a Markdown usando una estrategia robusta
///
/// Esta función realiza una conversión básica pero efectiva de HTML a Markdown,
/// removiendo scripts y estilos, y convirtiendo tags HTML comunes.
fn html_to_markdown(html: &str) -> String {
    let mut result = html.to_string();

    // Remover scripts y estilos
    result = remove_html_tags(&result, "script");
    result = remove_html_tags(&result, "style");

    // Conversiones de HTML a Markdown
    result = convert_headings(&result);
    result = convert_formatting(&result);
    result = convert_lists(&result);
    result = convert_code_blocks(&result);
    result = convert_links(&result);

    // Remover tags HTML restantes
    result = remove_remaining_html_tags(&result);

    // Limpiar espacios en blanco excesivos
    result = clean_whitespace(&result);

    result
}

/// Convierte headings HTML a Markdown
fn convert_headings(html: &str) -> String {
    let mut result = html.to_string();
    for level in 1..=6 {
        let open_tag = format!("<h{}>", level);
        let close_tag = format!("</h{}>", level);
        let markdown_prefix = "#".repeat(level);

        result = result.replace(&open_tag, &format!("\n{} ", markdown_prefix));
        result = result.replace(&close_tag, "\n");
    }
    result
}

/// Convierte formatos (negrita, cursiva, etc.) a Markdown
fn convert_formatting(html: &str) -> String {
    let mut result = html.to_string();

    result = result.replace("<strong>", "**");
    result = result.replace("</strong>", "**");
    result = result.replace("<b>", "**");
    result = result.replace("</b>", "**");
    result = result.replace("<em>", "*");
    result = result.replace("</em>", "*");
    result = result.replace("<i>", "*");
    result = result.replace("</i>", "*");
    result = result.replace("<u>", "");
    result = result.replace("</u>", "");
    result = result.replace("<p>", "\n");
    result = result.replace("</p>", "\n");
    result = result.replace("<br>", "\n");
    result = result.replace("<br/>", "\n");
    result = result.replace("<br />", "\n");

    result
}

/// Convierte listas HTML a Markdown
fn convert_lists(html: &str) -> String {
    let mut result = html.to_string();

    result = result.replace("<li>", "\n- ");
    result = result.replace("</li>", "");
    result = result.replace("<ul>", "");
    result = result.replace("</ul>", "");
    result = result.replace("<ol>", "");
    result = result.replace("</ol>", "");

    result
}

/// Convierte bloques de código HTML a Markdown
fn convert_code_blocks(html: &str) -> String {
    let mut result = html.to_string();

    result = result.replace("<code>", "`");
    result = result.replace("</code>", "`");
    result = result.replace("<pre>", "\n```\n");
    result = result.replace("</pre>", "\n```\n");

    result
}

/// Convierte enlaces HTML a Markdown
fn convert_links(html: &str) -> String {
    let mut result = String::new();
    let mut in_link = false;
    let mut link_text = String::new();
    let mut current_href = String::new();

    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            if chars.peek() == Some(&'a') {
                // Verificar si es un tag <a
                let mut tag = String::from("<a");
                while let Some(&c) = chars.peek() {
                    if c == '>' {
                        tag.push(chars.next().unwrap());
                        break;
                    }
                    tag.push(chars.next().unwrap());
                }

                // Extraer href
                if let Some(start) = tag.find("href=\"") {
                    let after_href = &tag[start + 6..];
                    if let Some(end) = after_href.find('"') {
                        current_href = after_href[..end].to_string();
                    }
                }

                in_link = true;
            } else if in_link && chars.peek() == Some(&'/') {
                // Es un </a>
                chars.next(); // consume '/'
                while let Some(&c) = chars.peek() {
                    if c == '>' {
                        chars.next();
                        break;
                    }
                    chars.next();
                }
                in_link = false;

                // Agregar el link en formato Markdown
                result.push('[');
                result.push_str(&link_text);
                result.push_str("](");
                result.push_str(&current_href);
                result.push(')');

                link_text.clear();
                current_href.clear();
            } else {
                result.push(ch);
            }
        } else if in_link && ch != '>' {
            link_text.push(ch);
        } else if !in_link {
            result.push(ch);
        }
    }

    result
}

/// Remueve las líneas que contienen un tag HTML específico
fn remove_html_tags(html: &str, tag: &str) -> String {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    let mut result = String::new();
    let mut skip = false;

    for line in html.lines() {
        if line.contains(&open_tag) {
            skip = true;
        }
        if line.contains(&close_tag) {
            skip = false;
            continue;
        }
        if !skip {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Remueve tags HTML restantes
fn remove_remaining_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(ch);
                }
            }
        }
    }

    result
}

/// Limpia espacios en blanco excesivos
fn clean_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
