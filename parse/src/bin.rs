use std::collections::HashMap;

use apt_capnp::binary;
use errors::*;
use fields;

use fill_dep;
use yes_no;

pub fn populate(mut output: binary::Builder, map: &mut HashMap<&str, &str>) -> Result<()> {
    {
        let mut builder = output.borrow().init_file();
        if let Some(s) = map.remove("Filename") {
            builder.set_name(s);
        }

        if let Some(s) = map.remove("Size") {
            builder.set_size(s.parse()?);
        }

        if let Some(s) = map.remove("MD5sum") {
            builder.set_md5(s);
        }

        if let Some(s) = map.remove("SHA1") {
            builder.set_sha1(s);
        }

        if let Some(s) = map.remove("SHA256") {
            builder.set_sha256(s);
        }

        if let Some(s) = map.remove("SHA512") {
            builder.set_sha512(s);
        }
    }

    if let Some(s) = map.remove("Essential") {
        output.set_essential(yes_no(s)?);
    }

    if let Some(s) = map.remove("Build-Essential") {
        output.set_build_essential((yes_no(s)?));
    }

    if let Some(s) = map.remove("Installed-Size") {
        output.set_installed_size(s.parse()?);
    }

    if let Some(text) = map.remove("Description") {
        if let Some(expected_md5) = map.remove("Description-md5") {
            ensure!(
                expected_md5 == format!("{:032x}", ::md5::compute(text.as_bytes())).as_str(),
                "Invalid md5 ({:?}) on Description",
                expected_md5
            );
        }
        output.set_description(text);
    }

    fill_dep(map, "Depends", |len| output.borrow().init_depends(len))?;

    fill_dep(
        map,
        "Recommends",
        |len| output.borrow().init_recommends(len),
    )?;

    fill_dep(map, "Suggests", |len| output.borrow().init_suggests(len))?;

    fill_dep(map, "Enhances", |len| output.borrow().init_enhances(len))?;

    fill_dep(
        map,
        "Pre-Depends",
        |len| output.borrow().init_pre_depends(len),
    )?;

    fill_dep(map, "Breaks", |len| output.borrow().init_breaks(len))?;

    fill_dep(map, "Conflicts", |len| output.borrow().init_conflicts(len))?;

    fill_dep(map, "Replaces", |len| output.borrow().init_replaces(len))?;

    fill_dep(map, "Provides", |len| output.borrow().init_provides(len))?;

    let mut unparsed = output.init_unparsed();

    for (key, val) in map.into_iter() {
        if fields::HANDLED_FIELDS_BINARY.contains(&key) {
            continue;
        }

        fields::set_field_binary(key, val, &mut unparsed)
            .chain_err(|| format!("setting extra field {}", key))?;
    }

    Ok(())
}
