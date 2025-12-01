# nbi - Name By Id

패키지 이름 가용성 체크 및 선점을 위한 TUI 도구입니다.

## 기능

### 검색 (Search)
다음 플랫폼에서 패키지/도메인 이름 가용성을 동시에 확인합니다:
- **npm** - `registry.npmjs.org` API
- **crates.io** - crates.io API
- **PyPI** - `pypi.org/simple` API
- **.dev 도메인** - DNS lookup

### 등록 (Register)
GitHub 레포지토리를 생성하여 이름을 선점합니다:
- 가용한 레지스트리에 대해 GitHub repo 생성
- 생성된 repo에서 각 레지스트리로 publish하여 이름 확보

## 설치

```bash
cargo install --path .
```

## 사용법

```bash
nbi
```

### 키보드 단축키

| 키 | 동작 |
|---|------|
| `q`, `Esc` | 종료 |
| `Tab` | 화면 전환 |
| `1` | 검색 화면 |
| `2` | 등록 화면 |
| `Enter` | 검색/등록 실행 |
| `↑/↓` | 결과 탐색 |
| `?` | 도움말 |

## GitHub 토큰 설정

등록 기능을 사용하려면 GitHub Personal Access Token이 필요합니다:

```bash
# 환경변수로 설정
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

토큰 생성: https://github.com/settings/tokens
- 필요 권한: `repo` 또는 `public_repo`

## API 정리

| 플랫폼 | Endpoint | 가용성 확인 |
|--------|----------|------------|
| npm | `GET https://registry.npmjs.org/{name}` | 404 = 미등록 |
| crates.io | `GET https://crates.io/api/v1/crates/{name}` | 404 = 미등록 |
| PyPI | `GET https://pypi.org/simple/{name}/` | 404 = 미등록 |
| .dev | DNS A record lookup | NXDOMAIN = 미등록 가능성 |
| GitHub | `POST https://api.github.com/user/repos` | 인증 필요 |

## 프로젝트 구조

```
src/
├── main.rs              # TUI 진입점 및 이벤트 핸들링
├── app.rs               # 앱 상태 관리
├── config.rs            # 설정 (GitHub 토큰 등)
├── ui/
│   ├── mod.rs           # UI 통합
│   ├── search.rs        # 검색 화면
│   └── register.rs      # 등록 화면
└── registry/
    ├── mod.rs           # 레지스트리 통합
    ├── npm.rs           # npm API
    ├── crates.rs        # crates.io API
    ├── pypi.rs          # PyPI API
    ├── domain.rs        # .dev 도메인 체크
    └── github.rs        # GitHub 레포 생성
```

## 의존성

- **ratatui** - TUI 프레임워크
- **crossterm** - 터미널 제어
- **tokio** - 비동기 런타임
- **reqwest** - HTTP 클라이언트
- **trust-dns-resolver** - DNS 조회

## 라이선스

MIT
