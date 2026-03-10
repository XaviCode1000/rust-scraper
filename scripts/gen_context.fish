#!/usr/bin/env fish

# Generate .github/context_map.json for AI agents (2026 Edition)
set -l output ".github/context_map.json"

# Architecture Map
set -l layers "Domain" "Application" "Infrastructure" "Adapters"

# Basic project metadata
set -l project "rust-scraper"
set -l timestamp (date -u +"%Y-%m-%dT%H:%M:%SZ")

echo "{" > $output
echo "  \"project\": \"$project\"," >> $output
echo "  \"timestamp\": \"$timestamp\"," >> $output
echo "  \"architecture\": \"Clean Architecture\"," >> $output
echo "  \"layers\": [" >> $output
for i in (seq (count $layers))
    set -l layer $layers[$i]
    if test $i -eq (count $layers)
        echo "    \"$layer\"" >> $output
    else
        echo "    \"$layer\"," >> $output
    end
end
echo "  ]," >> $output

# Dynamic File mapping using fd (as per user preference)
echo "  \"entry_points\": {" >> $output
echo "    \"lib\": \"src/lib.rs\"," >> $output
echo "    \"main\": \"src/main.rs\"" >> $output
echo "  }," >> $output

echo "  \"critical_paths\": [" >> $output
echo "    { \"path\": \"src/domain\", \"description\": \"Reglas de negocio puras. Sin IO.\" }," >> $output
echo "    { \"path\": \"src/application\", \"description\": \"Casos de uso y orquestación.\" }," >> $output
echo "    { \"path\": \"src/infrastructure\", \"description\": \"Implementaciones externas (HTTP, Persistence).\" }," >> $output
echo "    { \"path\": \"src/adapters\", \"description\": \"Interfaces de usuario (TUI/CLI).\" }" >> $output
echo "  ]" >> $output
echo "}" >> $output

echo "✅ [RUST-JARVIS] Context map generated successfully at $output"
chmod +x $output
