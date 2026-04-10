use crate::config::{RepoConfig, UserConfig};
use crate::utils::expand_path;

pub fn execute() -> Result<(), String> {
    let user_config = UserConfig::load()?;
    let repo_config = RepoConfig::from_user_cfg(&user_config)?;

    let target_path = std::path::Path::new(&user_config.repo_path);
    print_software_list(&repo_config, target_path);

    Ok(())
}

pub fn print_software_list(repo_config: &RepoConfig, target_path: &std::path::Path) {
    // 收集所有行数据
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut col_widths = [0usize; 5];

    let headers = ["名称", "链接模式", "状态", "仓库路径", "配置路径"];

    // 计算表头宽度
    for (i, h) in headers.iter().enumerate() {
        col_widths[i] = col_widths[i].max(display_width(h));
    }

    // 收集数据行
    for (name, software) in &repo_config.software {
        let link_mode = match software.link_mode {
            crate::config::LinkMode::Soft => "soft",
            crate::config::LinkMode::Hard => "hard",
            crate::config::LinkMode::Cp => "copy",
        };

        let src = target_path.join(&software.src_path);
        let status = if src.exists() { "✓" } else { "✗" };

        let config_path =
            software.get_config_path().map(|p| expand_path(&p)).unwrap_or_else(|| "-".to_string());

        let row = vec![
            name.clone(),
            link_mode.to_string(),
            status.to_string(),
            software.src_path.clone(),
            config_path,
        ];

        // 更新列宽
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(display_width(cell));
        }

        rows.push(row);
    }

    // 打印表格
    println!();

    // 顶部边框
    print_border(&col_widths, '╭', '┬', '╮', '─');

    // 表头
    print_row(&headers.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &col_widths);
    print_border(&col_widths, '├', '┼', '┤', '─');

    // 数据行
    for row in &rows {
        print_row(row, &col_widths);
    }

    // 底部边框
    print_border(&col_widths, '╰', '┴', '╯', '─');

    println!();
}

fn print_border(widths: &[usize; 5], left: char, mid: char, right: char, line: char) {
    print!("{}", left);
    for (i, w) in widths.iter().enumerate() {
        for _ in 0..(w + 2) {
            print!("{}", line);
        }
        if i < widths.len() - 1 {
            print!("{}", mid);
        }
    }
    println!("{}", right);
}

fn print_row(cells: &[String], widths: &[usize; 5]) {
    print!("│");
    for (i, (cell, width)) in cells.iter().zip(widths.iter()).enumerate() {
        let cell_width = display_width(cell);
        let padding = width - cell_width;
        print!(" {}{} ", cell, " ".repeat(padding));
        if i < widths.len() - 1 {
            print!("│");
        }
    }
    println!("│");
}

/// 计算字符串的显示宽度（使用 unicode-width 正确处理各种字符）
fn display_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}
