# Third-party notices

## libmpv (`libmpv-2.dll`)

Windows releases bundle `libmpv-2.dll` built by
[shinchiro/mpv-winbuild-cmake](https://github.com/shinchiro/mpv-winbuild-cmake).
That build includes GPL-licensed components, so the DLL is distributed under the
**GNU General Public License v3.0 or later**. Consequently, the *binary release archive as a whole*
is distributed under the terms of the GPL-3.0; the Rustpaper source code itself remains MIT-licensed.

- mpv source code: <https://github.com/mpv-player/mpv>
- Build scripts: <https://github.com/shinchiro/mpv-winbuild-cmake>
- License: <https://www.gnu.org/licenses/gpl-3.0.html>

## Inter font

Bundled in the Studio executable. Copyright the Inter Project Authors, licensed under the
[SIL Open Font License 1.1](assets/fonts/OFL.txt).

## Phosphor Icons

Bundled through the `egui-phosphor` crate. MIT License — <https://github.com/phosphor-icons/core>.

## Rust crates

All other dependencies are Rust crates listed in `Cargo.lock`, distributed under permissive
licenses (MIT, Apache-2.0, BSD, Zlib, ISC).
