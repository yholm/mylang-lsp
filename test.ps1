# mylang-lsp-startup.ps1

# Format the code
cargo fmt

# Build the project
cargo build

# Run the LSP with input from request.lsp and redirect output
Get-Content .\request.lsp -Raw | .\target\debug\mylang-lsp > .\result.lsp

# Remove the first line and save the rest into result.json
Get-Content .\result.lsp | Select-Object -Skip 1 | Set-Content .\result.json
