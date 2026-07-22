<p align="center">
  <img src="public/logo.png" width="128" alt="CDXTheme 로고">
</p>

<h1 align="center">CDXTheme</h1>

<p align="center">
  Codex와 ChatGPT를 나만의 모습으로 꾸미는 네이티브 데스크톱 테마 관리자.
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README.zh-CN.md">简体中文</a> ·
  <a href="README.ja.md">日本語</a> ·
  <strong>한국어</strong>
</p>

<p align="center">
  <a href="https://github.com/croath/CDX-Theme/releases/latest"><img src="https://img.shields.io/github/v/release/croath/CDX-Theme?style=flat-square&logo=github&label=release" alt="최신 릴리스"></a>
  <a href="https://github.com/croath/CDX-Theme/releases"><img src="https://img.shields.io/github/downloads/croath/CDX-Theme/total?style=flat-square&logo=github" alt="다운로드 수"></a>
  <a href="https://github.com/croath/CDX-Theme/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/croath/CDX-Theme/release.yml?style=flat-square&logo=githubactions&logoColor=white&label=release" alt="릴리스 빌드"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-555?style=flat-square&logo=apple" alt="macOS 및 Windows">
  <img src="https://img.shields.io/badge/Rust-1.96-orange?style=flat-square&logo=rust" alt="Rust 1.96">
  <img src="https://img.shields.io/badge/Tauri-2-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
  <a href="#라이선스"><img src="https://img.shields.io/badge/license-proprietary-lightgrey?style=flat-square" alt="독점 라이선스"></a>
</p>

> [!NOTE]
> CDXTheme는 독립적인 커뮤니티 프로젝트이며 OpenAI와 제휴 관계가 없고 공식적인 보증을 받지 않았습니다.

## CDXTheme 후원

CDXTheme는 독립적으로 유지 관리됩니다. 후원금은 지속적인 릴리스, 플랫폼 테스트, 테마 도구 개선 및 장기 유지 관리에 도움이 됩니다.

| 후원자 되기 | 비용 없이 돕기 |
| --- | --- |
| 후원 노출 및 협업을 받고 있습니다. 후원 문의는 [**@croath**에게 연락](https://github.com/croath)해 주세요. | [저장소에 Star 추가](https://github.com/croath/CDX-Theme), CDXTheme 공유, 버그 제보 또는 테마 기여. |

> 현재 공개된 결제 채널은 없습니다. 후원 관련 문의는 관리자에게 직접 연락해 주세요.

## CDXTheme 사용법

### 1. 다운로드

[GitHub Releases](https://github.com/croath/CDX-Theme/releases/latest)에서 최신 설치 파일을 받으세요.

| 플랫폼 | 패키지 | 상태 |
| --- | --- | --- |
| macOS 12+ (Apple Silicon) | `.dmg` | 지원 |
| Windows x64 | NSIS `.exe` | 지원 |
| Linux | — | 현재 지원 대상 아님 |

Codex / ChatGPT 데스크톱 앱이 설치되어 있어야 합니다. CDXTheme는 `127.0.0.1`의 Chrome DevTools Protocol(CDP)을 통해 앱과 로컬로 통신하며 기본 포트는 `9335`입니다.

### 2. 테마 선택 및 적용

1. **추천**을 열어 사용 가능한 테마와 설치된 테마를 살펴봅니다.
2. 원하는 테마를 선택하고 한 번의 클릭으로 적용합니다.
3. 요청이 표시되면 CDP 포트를 활성화하도록 Codex / ChatGPT 재실행을 허용합니다.

CDXTheme는 `~/.codex/config.toml`에서 지원되는 모양 설정을 업데이트하고 실시간 CSS 스킨을 데스크톱 렌더러에 주입합니다. 시작 시 읽는 모양 값이 실제로 변경된 경우에만 Codex를 다시 시작합니다.

### 3. 나만의 패키지 설치

**설치**를 열고 지원되는 휴대용 형식 중 하나를 가져옵니다.

| 확장자 | 패키지 `format` |
| --- | --- |
| `.cdxtheme` | `cdxtheme` |
| `.codedrobe-theme` | `codedrobe-theme` |

패키지는 스키마 버전 `1`을 사용하며 최대 크기는 **30 MB**입니다. `@import` 또는 `url(http…)`를 통한 원격 CSS 로드는 허용되지 않습니다. 여러 앱 대상을 포함할 수 있지만 현재 CDXTheme는 `targets.codex`만 적용합니다.

### 4. 기본 모양 복원

**복원**을 선택하면 최초 백업에서 관리되는 모양 값을 되돌리고 렌더러에 주입된 테마 요소를 제거합니다.

### 주요 기능

- 기본 제공, 온라인 및 로컬 설치 테마 탐색.
- 휴대용 테마 패키지 설치 및 삭제.
- 모양 설정과 실시간 CSS / 창 스킨을 함께 적용.
- Codex / ChatGPT를 이전에 관리되던 모양으로 복원.
- CDXTheme의 라이트, 다크 및 시스템 모양 전환.
- 앱에서 영어, 중국어 간체, 중국어 번체, 일본어 사용.
- CDP 포트를 설정하고 필요할 때 호스트 앱 재실행.

## 테마 제작 CLI

Rust CLI는 공유 `cdx-theme-core` 라이브러리의 간단한 명령줄 인터페이스입니다. 모든 옵션은 [전체 CLI 가이드](cli/README.md)를 참조하세요.

```bash
cargo install --path cli

# 소스 디렉터리를 휴대용 패키지로 묶기
cdxtheme theme pack path/to/theme-source

# 패키지 풀기 또는 변환
cdxtheme theme unpack theme.cdxtheme path/to/output
cdxtheme theme convert theme.codedrobe-theme

# CDP를 통해 패키지 직접 적용
cdxtheme apply --app codex --theme theme.cdxtheme
```

테마 소스 디렉터리는 `theme.json`(권장) 또는 `manifest.json`과 CSS, 선택적 이미지 자산으로 구성됩니다.

## 기술 개요

### 작동 방식

```text
                         ~/.codex/config.toml
                    ┌──────────────────────────► 시작 시 모양
                    │
┌──────────────┐    │    CDP on 127.0.0.1:9335
│   CDXTheme   │────┼──────────────────────────► 실시간 렌더러 스킨
│  Tauri app   │    │
└──────────────┘    └──────────────────────────► 백업 / 복원
```

1. **모양** — Codex 설정의 `[desktop]` 아래에서 선택된 키를 관리합니다.
2. **스킨** — CDP를 통해 패키지 CSS와 포함된 이미지를 `app://` 렌더러 대상에 주입합니다.
3. **복원** — `config.before.toml`에서 관리 대상 키를 복구하고 주입된 DOM을 제거합니다.
4. **업데이트** — 서명된 Tauri 업데이터 메타데이터를 확인하고 사용 가능한 릴리스를 설치합니다.

### 기술 스택 및 구조

| 계층 | 기술 | 역할 |
| --- | --- | --- |
| 데스크톱 셸 | Tauri 2 | 네이티브 창, 명령, 업데이트, 번들링 |
| 프런트엔드 | Rust · Leptos 0.8 · WASM | 클라이언트 UI 및 상태 |
| 스타일링 | Tailwind CSS 4 | 애플리케이션 UI 스타일 |
| 호스트 통합 | Rust · CDP | 실행, 주입, 검증 및 복원 |
| 빌드 | Cargo · Trunk · Bun | 워크스페이스, WASM 번들, 프런트엔드 의존성 |

```text
├── src/          # Leptos CSR 프런트엔드
├── app-tauri/    # Tauri 백엔드 및 데스크톱 번들
├── core/         # 공유 패키지, 실행, 적용 및 주입 로직
├── cli/          # cdxtheme 테마 제작 CLI
├── types/        # 공유 테마 타입
├── assets/       # 렌더러 주입 스크립트
├── public/       # 정적 자산
├── style/        # Tailwind 진입점
└── scripts/      # 빌드 및 선택적 도우미 스크립트
```

### 개발

[Rust](https://rustup.rs/) `1.96.0`, `wasm32-unknown-unknown` 대상, [Trunk](https://trunkrs.dev/), Tauri CLI 2, Bun 또는 Node가 필요합니다. macOS 개발에는 Xcode Command Line Tools가, Windows에는 WebView2가 추가로 필요합니다.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli --version "^2"
bun install
cargo tauri dev
```

Trunk는 `http://localhost:1420`에서 프런트엔드를 제공합니다. 디버그 빌드는 터미널과 플랫폼 앱 로그 디렉터리에 로그를 기록하고 Web Inspector를 자동으로 엽니다.

유용한 검사 명령:

```bash
cargo check --manifest-path app-tauri/Cargo.toml
cargo check --target wasm32-unknown-unknown
cargo test --manifest-path app-tauri/Cargo.toml --lib
```

### 빌드

```bash
# macOS / Linux 호스트
./scripts/build.sh
./scripts/build.sh --debug
./scripts/build.sh --check

# Tauri 직접 실행
cargo tauri build --manifest-path app-tauri/Cargo.toml
```

```powershell
# Windows PowerShell
.\scripts\build.ps1
.\scripts\build.ps1 -Debug
.\scripts\build.ps1 -Check
```

번들은 `target/release/bundle/` 아래에 생성됩니다. GitHub Release를 게시하면 Apple Silicon macOS 및 Windows x64 결과물을 만드는 릴리스 워크플로가 실행됩니다.

### 기본값 및 경로

| 항목 | 기본값 / 경로 |
| --- | --- |
| CDP 엔드포인트 | `127.0.0.1:9335` |
| Codex 설정 | `~/.codex/config.toml` |
| Windows Codex 설정 | `%USERPROFILE%\.codex\config.toml` |
| 최초 적용 백업 | 앱 데이터 디렉터리 → `config.before.toml` |
| 사용자 테마 | 앱 로컬 데이터 디렉터리 → `themes/` |

## 문제 해결

<details>
<summary><strong>Codex / ChatGPT를 찾을 수 없음</strong></summary>

먼저 데스크톱 앱을 설치하세요. Windows에서는 `OpenAI.Codex`라는 Microsoft Store 패키지도 감지합니다.
</details>

<details>
<summary><strong>CDP 연결 끊김</strong></summary>

**설정**을 열어 포트를 확인한 뒤 저장하고 다시 실행하세요. CDXTheme와 호스트 앱은 사용 가능한 동일한 포트를 사용해야 합니다.
</details>

<details>
<summary><strong>모양 또는 스킨이 업데이트되지 않음</strong></summary>

시작 시 모양 값은 호스트를 다시 시작해야 하며 실시간 CSS에는 CDP 연결이 필요합니다. 연결 상태를 확인한 후 테마를 다시 적용하세요.
</details>

## 라이선스

별도 명시가 없는 한 이 프로젝트는 작성자가 제공하는 독점 조건을 따릅니다. 타사 구성 요소에는 각 구성 요소의 라이선스가 적용됩니다.

---

<p align="center">
  <a href="https://github.com/croath/CDX-Theme/releases/latest">다운로드</a> ·
  <a href="https://github.com/croath/CDX-Theme/issues">이슈</a> ·
  <a href="cli/README.md">CLI 문서</a> ·
  <a href="https://github.com/croath">후원 문의</a>
</p>
