use errors::*;

use std::fs;
use std::io;
use std::path;

use std::io::BufRead;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub src: bool,
    pub url: String,
    pub suite_codename: String,
    pub components: Vec<String>,
    pub arch: Option<String>,
}

fn read_single_line(line: &str) -> Result<Vec<Entry>> {
    let line = match line.find('#') {
        Some(comment) => &line[..comment],
        None => line,
    }.trim();

    if line.is_empty() {
        return Ok(Vec::new());
    }

    let mut parts = line.split_whitespace().peekable();

    let src = parts.next().ok_or("deb{,s,-src} section required")?;
    let arch = match parts.peek() {
        Some(&val) if val.starts_with("[") => {
            parts.next();
            Some(val)
        }
        Some(_) => None,
        None => bail!("unexpected end of line looking for arch or url"),
    };

    let url = parts.next().ok_or("url section required")?;
    let suite = parts.next().ok_or("suite section required")?;

    let components: Vec<&str> = parts.collect();

    let srcs: &[bool] = match src {
        "deb" => &[false],
        "deb-src" => &[true],
        "debs" => &[false, true],
        other => bail!("unsupported deb-src tag: {:?}", other),
    };

    let mut ret = Vec::with_capacity(srcs.len());

    for src in srcs {
        ret.push(Entry {
            src: *src,
            url: if url.ends_with('/') {
                url.to_string()
            } else {
                format!("{}/", url)
            },
            suite_codename: suite.to_string(),
            components: components.iter().map(|x| x.to_string()).collect(),
            arch: arch.map(|arch| arch.to_string()),
        });
    }

    Ok(ret)
}

fn read_single_line_number(line: &str, no: usize) -> Result<Vec<Entry>> {
    read_single_line(line).chain_err(|| format!("parsing line {}", no + 1))
}

pub fn read(from: &str) -> Result<Vec<Entry>> {
    from.lines()
        .enumerate()
        .map(|(no, line)| read_single_line_number(line, no))
        .collect::<Result<Vec<Vec<Entry>>>>()
        .map(|vec_vec| vec_vec.into_iter().flat_map(|x| x).collect())
}

pub fn load<P: AsRef<path::Path>>(path: P) -> Result<Vec<Entry>> {
    io::BufReader::new(fs::File::open(path)?)
        .lines()
        .enumerate()
        .map(|(no, line)| match line {
            Ok(line) => read_single_line_number(&line, no),
            Err(e) => Err(Error::with_chain(e, format!("reading around line {}", no))),
        })
        .collect::<Result<Vec<Vec<Entry>>>>()
        .map(|vec_vec| vec_vec.into_iter().flat_map(|x| x).collect())
}

#[cfg(test)]
mod tests {
    use super::read;
    use super::Entry;

    #[test]
    fn simple() {
        assert_eq!(
            vec![
                Entry {
                    src: false,
                    arch: None,
                    url: "http://foo/".to_string(),
                    suite_codename: "bar".to_string(),
                    components: vec!["baz".to_string(), "quux".to_string()],
                },
                Entry {
                    src: true,
                    arch: None,
                    url: "http://foo/".to_string(),
                    suite_codename: "bar".to_string(),
                    components: vec!["baz".to_string(), "quux".to_string()],
                },
            ],
            read(
                r"
deb     http://foo  bar  baz quux
deb-src http://foo  bar  baz quux
",
            ).unwrap()
        );
    }
}
