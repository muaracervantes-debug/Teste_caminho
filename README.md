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

- [x] Estrutura inicial Cargo.
- [x] Ciclo de vida da janela com winit.
- [x] Inicialização do dispositivo e da superfície wgpu.
- [x] Pipeline gráfico com shader WGSL.
- [x] Teste de renderização 3D com cubo, buffer de profundidade e câmera inicial.
- [x] Compilação e execução no Windows.

No Windows, a inicialização gráfica utiliza Direct3D 12. O cubo é somente uma geometria temporária para validar o pipeline e será substituído pelo terreno nas próximas etapas.

### Etapa 2 — Câmera 3D

- [x] Câmera em perspectiva com ângulo superior.
- [x] Movimentação horizontal com WASD ou setas.
- [x] Rotação horizontal com Q/E.
- [x] Zoom com a roda do mouse.
- [x] Atualização da matriz de câmera conforme entrada e redimensionamento.
- [x] Limites de zoom e testes unitários básicos.

Controles: WASD/setas movem a câmera, Q/E giram a visão, roda do mouse aproxima/afasta e Esc fecha a aplicação.

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
├── app.rs             # Janela, eventos e ciclo de vida da aplicação
├── camera.rs          # Câmera 3D e controles
├── diagnostics.rs     # Registro de erros e panics
├── main.rs            # Ponto de entrada
├── renderer.rs        # Inicialização e renderização wgpu
└── shaders/
    └── basic.wgsl     # Shader inicial
```

Os módulos de mundo, chunks, geração procedural, obstáculos e navegação serão adicionados conforme as etapas forem implementadas, evitando criar abstrações antes de existir uma necessidade real.

## Próximas etapas

1. [x] Câmera 3D e controles.
2. Terreno básico com relevo.
3. Divisão em chunks.
4. Geração procedural reproduzível por seed.
5. Carregamento e descarregamento de chunks.
6. Obstáculos editáveis.
7. Dados independentes para navegação.
8. Diagnóstico, testes e otimizações.

Cada etapa deverá ser validada antes do início da seguinte.
