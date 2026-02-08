# Build Notes - Live Server RS

## Alterações no Workflow GitHub Actions

O workflow foi otimizado para resolver problemas de compilação cruzada:

### ✅ Principais Melhorias

1. **Uso de `cross` para compilação cruzada**
   - Targets ARM (aarch64, armv7) agora usam `cross` em vez de tentar compilação nativa
   - Evita conflitos de arquitetura entre diferentes targets no mesmo sistema

2. **Cache separado por target**
   - Cada target tem sua própria chave de cache
   - Previne conflitos de arquivos compilados para diferentes arquiteturas

3. **Testes apenas em nativos**
   - Testes são executados apenas em x86_64 em cada SO (Linux, Windows, macOS)
   - Targets cruzados pulam testes (não podem ser executados no host)

4. **Nomes únicos de artifacts**
   - Cada build resultará em um artifact único:
     - `live_server_rs-linux-x86_64`
     - `live_server_rs-linux-aarch64`
     - `live_server_rs-linux-armv7`
     - `live_server_rs-linux-i686`
     - `live_server_rs-windows-x86_64.exe`
     - `live_server_rs-windows-i686.exe`
     - `live_server_rs-windows-aarch64.exe`
     - `live_server_rs-macos-x86_64`
     - `live_server_rs-macos-aarch64`

### 🏗️ Build Matrix

| OS | Target | Método | Tipo |
|----|--------|--------|------|
| ubuntu-latest | x86_64-unknown-linux-gnu | cargo | nativo |
| ubuntu-latest | aarch64-unknown-linux-gnu | cross | cross-compile |
| ubuntu-latest | armv7-unknown-linux-gnueabihf | cross | cross-compile |
| ubuntu-latest | i686-unknown-linux-gnu | cross | cross-compile |
| windows-latest | x86_64-pc-windows-msvc | cargo | nativo |
| windows-latest | i686-pc-windows-msvc | cargo | nativo |
| windows-latest | aarch64-pc-windows-msvc | cargo | nativo |
| macos-latest | x86_64-apple-darwin | cargo | nativo |
| macos-latest | aarch64-apple-darwin | cargo | nativo |

### 📦 Release Automática

Quando você criar uma tag (ex: `v1.0.0`), o workflow:
1. Constrói todos os 9 binários
2. Cria uma release no GitHub
3. Anexa todos os binários à release

Exemplo:
```bash
git tag v1.0.0
git push origin v1.0.0
```

### 🔧 Como Usar Localmente

Para testar cross-compilation localmente:

```bash
# Instalar cross
cargo install cross

# Compilar para aarch64
cross build --release --target aarch64-unknown-linux-gnu

# Compilar para armv7
cross build --release --target armv7-unknown-linux-gnueabihf

# Compilar para i686
cross build --release --target i686-unknown-linux-gnu
```

### 📝 Notas de Compatibilidade

- **Linux x86_64**: Suporta qualquer distribuição moderna
- **Linux aarch64**: Para ARM 64-bit (Raspberry Pi 4 64-bit, etc)
- **Linux armv7**: Para ARM 32-bit (Raspberry Pi 3, etc)
- **Linux i686**: Para Intel 32-bit legado
- **Windows**: MSVC runtime
- **macOS**: Suporta Intel e Apple Silicon (M1/M2/M3)

