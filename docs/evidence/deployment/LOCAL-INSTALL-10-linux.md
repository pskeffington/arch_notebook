# LOCAL-INSTALL-10 · current-user launcher synchronization

- **Task / owner:** Install the current Rust release build and synchronize the user's launcher, case-file association, SQL scheduler helper, and graphics; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product source:** `7811d5256b50bc319c7c78b2a5e6e68a76ef41bc`.
- **Host:** Linux x86_64; current-user install only, no `sudo` or system-wide package changes.
- **Release and installed executable SHA-256:** `979aeb9aef259dbbc410088f96a5241da659f07998e7d8e011ca71b924042976`.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Current release installed | **PASS** | `scripts/install-linux-user.sh /tmp/arch-notebook-sync-7811d52/arch-notebook-linux-x86_64` exited 0. Installed `/home/sysadmin/.local/bin/arch-notebook` matches the release binary. `--version --json` returned `{"name":"arch-notebook","schema_version":1,"version":"0.1.0"}`. |
| Menu entry and Desktop shortcut | **PASS** | `desktop-file-validate` exited 0 for both entries. They compare byte-for-byte; SHA-256 `0997476bbec189ce6c3a98c22670cd4b8aebd989c721bcced4bf6eff5c141f4c`. |
| Application graphics | **PASS** | Installed SVG and 1024px PNG compare byte-for-byte with `assets/icons/`; SVG SHA-256 `2807b09bd0b4cd13ebeeb58620ba8ca60330dae8bff313c0f32b1b5117fd77e8`, PNG SHA-256 `c5de11f0910852848de318e335e59e198b289c9a4260457679401060995eca88`. |
| Automation helper | **PASS** | Installed `arch-notebook-install-automation` compares byte-for-byte with `scripts/install-linux-automation.sh`; SHA-256 `a04d9a7cf11746058b4862e5616f63c165669deac3ef8aba17edc79dc617dbf8`. |
| Case-file association | **PASS** | `gio mime application/vnd.arch-notebook.case+json` lists `arch-notebook.desktop` as default and recommended. Installed MIME definition matches the repository file; SHA-256 `c5086e63e8cdbf0b8e3a43e359227bbec1e5936c7079abe0c9bad65d52e108db`. |
| Native application launch and current UI review | **NOT RUN** | This check verifies installation and launcher assets only. It does not establish native interaction or visual acceptance. |

The install replaced only the current user's application files. It did not open or modify a case file or SQL source, install a scheduled job, or change system-wide packages.
