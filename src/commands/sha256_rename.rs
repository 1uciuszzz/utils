use anyhow::{Context, Result, bail};
use clap::Args;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// 计算文件 SHA256 并将文件重命名为其哈希值
#[derive(Args, Debug)]
pub struct Sha256RenameArgs {
    /// 要处理的文件路径，支持通配符（如 *.jpg、**/*.png）
    #[arg(required = true, value_name = "FILE")]
    pub files: Vec<String>,

    /// 去掉原始文件扩展名（默认保留）
    #[arg(short = 'E', long = "no-ext")]
    pub no_ext: bool,

    /// 仅计算并打印哈希值，不重命名文件
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// 输出目标目录（默认与原文件同目录）
    #[arg(short = 'o', long = "output-dir", value_name = "DIR")]
    pub output_dir: Option<PathBuf>,
}

pub fn run(args: &Sha256RenameArgs) -> Result<()> {
    if let Some(dir) = &args.output_dir {
        if !dir.exists() {
            bail!("输出目录不存在: {}", dir.display());
        }
        if !dir.is_dir() {
            bail!("指定的输出路径不是目录: {}", dir.display());
        }
    }

    let paths = expand_globs(&args.files)?;
    if paths.is_empty() {
        bail!("没有匹配到任何文件");
    }

    for path in &paths {
        if let Err(e) = process_file(path, args) {
            eprintln!("错误 [{}]: {:#}", path.display(), e);
        }
    }
    Ok(())
}

/// 展开通配符，返回所有匹配的文件路径（去重、排序）
fn expand_globs(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for pattern in patterns {
        let matched: Vec<PathBuf> = glob::glob(pattern)
            .with_context(|| format!("无效的通配符模式: {}", pattern))?
            .filter_map(|entry| match entry {
                Ok(p) if p.is_file() => Some(p),
                Ok(_) => None, // 跳过目录
                Err(e) => {
                    eprintln!("警告: {}", e);
                    None
                }
            })
            .collect();

        if matched.is_empty() {
            // 无匹配时作为字面路径处理，保持与普通用法一致
            paths.push(PathBuf::from(pattern));
        } else {
            paths.extend(matched);
        }
    }
    // 去重，保持顺序
    paths.dedup();
    Ok(paths)
}

fn sha256_of_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("无法打开文件: {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn process_file(path: &Path, args: &Sha256RenameArgs) -> Result<()> {
    if !path.exists() {
        bail!("文件不存在: {}", path.display());
    }
    if !path.is_file() {
        bail!("路径不是文件: {}", path.display());
    }

    let hash = sha256_of_file(path)?;

    let new_name = if args.no_ext {
        hash.clone()
    } else {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => format!("{}.{}", hash, ext),
            None => hash.clone(),
        }
    };

    let parent = args
        .output_dir
        .as_deref()
        .or_else(|| path.parent())
        .unwrap_or_else(|| Path::new("."));

    let new_path = parent.join(&new_name);

    if args.dry_run {
        println!("{}  {}  ->  {}", hash, path.display(), new_path.display());
    } else {
        if new_path.exists() {
            eprintln!(
                "警告: 目标已存在，跳过 [{}] -> [{}]",
                path.display(),
                new_path.display()
            );
            return Ok(());
        }
        fs::rename(path, &new_path).with_context(|| {
            format!(
                "重命名失败: {} -> {}",
                path.display(),
                new_path.display()
            )
        })?;
        println!("{}", new_path.display());
    }

    Ok(())
}
