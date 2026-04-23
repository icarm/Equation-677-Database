use crate::*;

pub fn load_file(file: &str) {
    let ss = &format!("File('{file}')");
    for m in magmas_from_file(file) {
        present_model(m.n, &ss, |x, y| m.f(x, y));
    }
}

pub fn magma_from_file(file: &str) -> MatrixMagma {
    let v = magmas_from_file(file);
    v.into_iter().next().unwrap()
}

pub fn magmas_from_file(file: &str) -> Vec<MatrixMagma> {
    let mut s = std::fs::read_to_string(file).unwrap();
    let mut out = Vec::new();

    let mut current = String::new();
    for line in s.split("\n") {
        if line.chars().all(|x| x.is_whitespace() || x.is_digit(10) || x == '-') && line.trim().len() > 4 {
            current.push_str(&line);
            current.push('\n');
        } else {
            current = current.trim().to_string();
            if !current.is_empty() {
                let m = MatrixMagma::parse(&current);
                out.push(m);
                current = String::new();
            }
        }
    }
    out
}
