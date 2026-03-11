//! Quick test of AI pipeline
use rust_scraper::infrastructure::ai::{SemanticCleanerImpl, ModelConfig};
use rust_scraper::SemanticCleaner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing AI Pipeline...");
    
    let config = ModelConfig::default();
    println!("📦 Model config created");
    
    let cleaner = SemanticCleanerImpl::new(config).await?;
    println!("✅ SemanticCleaner created");
    
    let html = r#"<html><body>
<h1>Hello World</h1>

<p>This is a test paragraph with some content. It has multiple sentences. This is the third sentence in the first paragraph.</p>

<p>Another paragraph with more information. This is the second sentence. And this is another sentence.</p>

<p>Third paragraph here with even more text for better chunking. More text here to ensure chunking works.</p>

<div>
    <p>Fourth paragraph inside a div. This has content too. And more content.</p>
</div>

<main>
    <article>
        <h2>Article Title</h2>
        <p>First paragraph of the article with meaningful content. This is the first sentence. This is the second sentence.</p>
        <p>Second paragraph continues the discussion with more details. More details here. Even more details.</p>
        <p>Third paragraph provides additional information and context. More information. Even more information.</p>
    </article>
</main>

</body></html>"#;
    
    println!("📄 HTML length: {} chars", html.len());
    
    println!("🔄 Cleaning HTML...");
    let chunks = cleaner.clean(html).await?;
    
    println!("✅ Generated {} chunks", chunks.len());
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {} chars, has_embeddings: {}", 
            i, 
            chunk.content.len(),
            chunk.has_embeddings()
        );
    }
    
    println!("\n🎉 AI Pipeline works!");
    Ok(())
}
