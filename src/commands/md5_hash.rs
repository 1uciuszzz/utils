use anyhow::Result;
use clap::Args;

/// 计算字符串的 MD5 哈希值
#[derive(Args, Debug)]
pub struct Md5HashArgs {
    /// 要计算哈希的字符串
    #[arg(required = true, value_name = "TEXT")]
    pub text: String,

    /// 输出完整的 32 位 MD5（默认输出 16 位，取第 9-24 位字符）
    #[arg(short = 'f', long = "full")]
    pub full: bool,

    /// 输出大写形式
    #[arg(short = 'u', long = "upper")]
    pub upper: bool,
}

pub fn run(args: &Md5HashArgs) -> Result<()> {
    let digest = md5::compute(args.text.as_bytes());
    let hex_full = format!("{:x}", digest);

    // 16 位取中间 8 字节（第 9-24 位字符，与 Java DigestUtils 行为一致）
    let result = if args.full {
        hex_full.as_str().to_owned()
    } else {
        hex_full[8..24].to_owned()
    };

    let result = if args.upper {
        result.to_uppercase()
    } else {
        result
    };

    println!("{}", result);
    Ok(())
}
