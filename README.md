# Teste Caminho — Protótipo de mundo 3D

Protótipo independente em Rust para desenvolver e validar a estrutura de um mundo 3D procedural, com relevo, chunks e obstáculos. O projeto será preparado para receber futuramente sistemas de navegação e movimentação de entidades, mas **não terá zumbis, personagens ou IA nesta etapa**.

## Tecnologias

- Rust e Cargo
- winit para janela e entrada
- wgpu para renderização 3D
- WGSL para shaders
- glam para matemática 3D

Não são utilizadas engines de jogos.

## Estado atual

### Etapa 1 — Fundação gráfica

Em desenvolvimento na branch `feat/phase-01-foundation`.

- [x] Estrutura inicial Cargo.
- [x] Ciclo de vida da janela com winit.
- [x] Inicialização do dispositivo e da superfície wgpu.
- [x] Pipeline gráfico com shader WGSL.
- [x] Teste de renderização 3D com cubo, buffer de profundidade e câmera fixa.
- [ ] Validar compilação e execução no Windows.

O cubo é somente uma geometria temporária para validar o pipeline. Ele será substituído pelo terreno nas próximas etapas.

## Executar no Windows

Requisitos:

- Windows 10 ou superior.
- Rust instalado pelo [rustup](https://rustup.rs/).
- Driver gráfico atualizado com suporte a Direct3D 12 ou Vulkan.

Na pasta do projeto:

```powershell
cargo check
cargo run
```

Para compilar a versão otimizada:

```powershell
cargo build --release
```

## Organização inicial

```text
src/
├── app.rs             # Janela e ciclo de vida da aplicação
├── main.rs            # Ponto de entrada
├── renderer.rs        # Inicialização e renderização wgpu
└── shaders/
    └── basic.wgsl     # Shader inicial
```

Os módulos de mundo, chunks, geração procedural, obstáculos e navegação serão adicionados conforme as etapas forem implementadas, evitando criar abstrações antes de existir uma necessidade real.

## Próximas etapas

1. Câmera 3D e controles.
2. Terreno básico com relevo.
3. Divisão em chunks.
4. Geração procedural reproduzível por seed.
5. Carregamento e descarregamento de chunks.
6. Obstáculos editáveis.
7. Dados independentes para navegação.
8. Diagnóstico, testes e otimizações.

Cada etapa deverá ser validada antes do início da seguinte.
