//! Build.rs

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use macro_toolset::{str_concat, string::b64_padding};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=assets/theme");
    println!("cargo:rerun-if-changed=build.rs");

    gen_server_version()?;

    let moe_counter_themes = [
        "3d-num",
        "ai-1",
        "asoul",
        "booru-ffsr",
        "booru-helltaker",
        "booru-huggboo",
        "booru-jaypee",
        "booru-koe",
        "booru-lewd",
        "booru-lisu",
        "booru-mjg",
        "booru-mof",
        "booru-nandroid",
        "booru-qualityhentais",
        "booru-r6gdrawfriends",
        "booru-rfck",
        "booru-smtg",
        "booru-snyde",
        "booru-the-collection",
        "booru-touhoulat",
        "booru-townofgravityfalls",
        "booru-twifanartsfw",
        "booru-ve",
        "booru-vivi",
        "booru-vp",
        "booru-yuyuyui",
        "capoo-1",
        "capoo-2",
        "e621",
        "food",
        "gelbooru",
        "gelbooru-h",
        "green",
        "kasuterura-1",
        "kasuterura-2",
        "kasuterura-3",
        "kasuterura-4",
        "kyun",
        "love-and-deepspace",
        "minecraft",
        "moebooru",
        "moebooru-h",
        "morden-num",
        "nixietube-1",
        "nixietube-2",
        "normal-1",
        "normal-2",
        "original-new",
        "original-old",
        "rule34",
        "shimmie2",
        "sketch-1",
        "sketch-2",
    ];

    let mut data = Vec::with_capacity(moe_counter_themes.len());

    for moe_counter_theme in moe_counter_themes {
        let (data_url_prefix, ext) = {
            let (image_path_png, image_path_gif): (PathBuf, PathBuf) = (
                format!("./assets/theme/{moe_counter_theme}/0.png").into(),
                format!("./assets/theme/{moe_counter_theme}/0.gif").into(),
            );

            match (image_path_png.exists(), image_path_gif.exists()) {
                (true, false) => ("data:image/png;base64,", "png"),
                (false, true) => ("data:image/gif;base64,", "gif"),
                _ => {
                    bail!("No image or multiple images found for {moe_counter_theme}")
                }
            }
        };

        let (mut size_w, mut size_h) = (0, 0);

        let (image_start_path, image_end_path): (PathBuf, PathBuf) = (
            format!("./assets/theme/{moe_counter_theme}/_start.{ext}").into(),
            format!("./assets/theme/{moe_counter_theme}/_end.{ext}").into(),
        );

        data.push((
            moe_counter_theme,
            (0..=9)
                .map(|i| {
                    let image_data =
                        fs::read(format!("./assets/theme/{moe_counter_theme}/{i}.{ext}"))
                            .expect("Read image error");

                    let image_size =
                        imagesize::blob_size(&image_data).expect("Read image data error");

                    size_w = size_w.max(image_size.width);
                    size_h = size_h.max(image_size.height);

                    str_concat!(data_url_prefix, b64_padding::STANDARD::encode(&image_data))
                })
                .collect::<Vec<_>>(),
            if image_start_path.exists() {
                let image_data = fs::read(image_start_path).expect("Read image _end error");

                let image_size = imagesize::blob_size(&image_data).expect("Read image data error");

                size_w = size_w.max(image_size.width);
                size_h = size_h.max(image_size.height);

                Some(str_concat!(
                    data_url_prefix,
                    b64_padding::STANDARD::encode(&image_data)
                ))
            } else {
                None
            },
            if image_end_path.exists() {
                let image_data = fs::read(image_end_path).expect("Read image _end error");

                let image_size = imagesize::blob_size(&image_data).expect("Read image data error");

                size_w = size_w.max(image_size.width);
                size_h = size_h.max(image_size.height);

                Some(str_concat!(
                    data_url_prefix,
                    b64_padding::STANDARD::encode(&image_data)
                ))
            } else {
                None
            },
            (size_w, size_h),
        ));
    }

    let out_dir = env::var_os("OUT_DIR").context("No OUT_DIR")?;

    let mut dest_file = Vec::new();

    let _ = writeln!(&mut dest_file, "mod moe_counter_list {{");
    let _ = writeln!(
        &mut dest_file,
        "    #![allow(non_upper_case_globals, reason = \"generated\")]"
    );
    let _ = writeln!(&mut dest_file);

    // THEMES_LIST
    let _ = writeln!(
        &mut dest_file,
        "    pub(super) static THEMES_LIST: [&str; {}] = {moe_counter_themes:?};",
        moe_counter_themes.len()
    );
    let _ = writeln!(&mut dest_file);

    // STATIC CONTENT

    for (name, data, start, end, (size_w, size_h)) in &data {
        let name = name.replace('-', "_");
        let _ = writeln!(
            &mut dest_file,
            "    static THEME_{name}: [&str; 10] = {data:?};",
        );
        let _ = writeln!(
            &mut dest_file,
            "    static THEME_{name}_SIZE_W: usize = {size_w};",
        );
        let _ = writeln!(
            &mut dest_file,
            "    static THEME_{name}_SIZE_H: usize = {size_h};",
        );
        let _ = writeln!(
            &mut dest_file,
            "    static THEME_{name}_START: Option<&str> = {start:?};",
        );
        let _ = writeln!(
            &mut dest_file,
            "    static THEME_{name}_END: Option<&str> = {end:?};",
        );
    }

    let _ = writeln!(&mut dest_file);
    let _ = writeln!(&mut dest_file, "    pub(super) struct MoeCounter {{");
    let _ = writeln!(
        &mut dest_file,
        "        pub pics: &'static [&'static str; 10],"
    );
    let _ = writeln!(
        &mut dest_file,
        "        pub pic_start: Option<&'static str>,"
    );
    let _ = writeln!(&mut dest_file, "        pub pic_end: Option<&'static str>,");
    let _ = writeln!(&mut dest_file, "        pub size_w: usize,");
    let _ = writeln!(&mut dest_file, "        pub size_h: usize,");
    let _ = writeln!(&mut dest_file, "    }}");

    let _ = writeln!(&mut dest_file);
    let _ = writeln!(&mut dest_file, "    #[inline]");
    let _ = writeln!(
        &mut dest_file,
        "    #[allow(unused, reason = \"generated\")]"
    );
    let _ = writeln!(
        &mut dest_file,
        "    pub(super) fn moe_counter_list(name: impl AsRef<[u8]>) -> MoeCounter {{"
    );

    let _ = writeln!(&mut dest_file, "        match name.as_ref() {{");

    for (name, _, _, _, _) in &data {
        let _ = writeln!(
            &mut dest_file,
            "            b\"{name}\" => MoeCounter {{ pics: &THEME_{r_name}, pic_start: \
             THEME_{r_name}_START, pic_end: THEME_{r_name}_END, size_w: THEME_{r_name}_SIZE_W, \
             size_h: THEME_{r_name}_SIZE_H }},",
            r_name = name.replace('-', "_")
        );
    }

    let _ = writeln!(
        &mut dest_file,
        "            _ => MoeCounter {{ pics: &THEME_{r_name}, pic_start: THEME_{r_name}_START, \
         pic_end: THEME_{r_name}_END, size_w: THEME_{r_name}_SIZE_W, size_h: \
         THEME_{r_name}_SIZE_H }},",
        r_name = &data[0].0.replace('-', "_")
    );
    let _ = writeln!(&mut dest_file, "        }}");
    let _ = writeln!(&mut dest_file, "    }}");
    let _ = writeln!(&mut dest_file, "}}");

    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(Path::new(&out_dir).join("moe-counter.rs"))
        .context("Write generated `moe-counter.rs` error")?
        .write_all(&dest_file)
        .context("Write generated `moe-counter.rs` error")
}

/// Gen server version
fn gen_server_version() -> Result<()> {
    let main_version = env!("CARGO_PKG_VERSION");
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())
        .unwrap();
    let commit = Command::new("git")
        .args(["describe", "--always"])
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())
        .unwrap();
    let release_mode = if cfg!(debug_assertions) || cfg!(test) {
        "DEBUG"
    } else {
        "RELEASE"
    };
    let version =
        format!("{}-{}-{}-{}", main_version, branch, commit, release_mode).replace('\n', "");
    File::create(Path::new(&env::var("OUT_DIR")?).join("VERSION"))?
        .write_all(version.trim().as_bytes())?;

    let now = chrono::Local::now().to_rfc3339();
    File::create(Path::new(&env::var("OUT_DIR")?).join("BUILD_TIME"))?
        .write_all(now.trim().as_bytes())?;

    Ok(())
}
