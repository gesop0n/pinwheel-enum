use clap::{Parser, Subcommand, ValueEnum};
use pinwheel_core::instance::PinwheelInstance;
use pinwheel_core::labeled::enumerate_labeled;
use pinwheel_core::quotient::enumerate_schedules;

/// 輪番割当（pinwheel scheduling）のスケジュール列挙 CLI
#[derive(Parser)]
#[command(name = "pinwheel", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// 列挙結果の表示形式
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    /// タスク文字 A,B,C で表示（holonomy 展開済みの具体的な日割）
    Tasks,
    /// 周期値の列で表示（商グラフの単純閉路。同周期タスクは区別しない）
    Periods,
}

#[derive(Subcommand)]
enum Command {
    /// スケジュールを列挙する
    Enumerate {
        /// 周期列（例: 2 4 4）
        #[arg(required = true, num_args = 1..)]
        periods: Vec<u32>,
        /// 表示形式: tasks = A,B,C のラベル付き / periods = 周期列
        #[arg(long, value_enum, default_value_t = Format::Tasks)]
        format: Format,
        /// 件数だけ表示する
        #[arg(long)]
        count: bool,
    },
    /// 密度 D(A) = Σ 1/a_i を表示する
    Density {
        /// 周期列（例: 2 4 4）
        #[arg(required = true, num_args = 1..)]
        periods: Vec<u32>,
    },
}

/// タスク添字（0,1,2,…）を A,B,C,… に変換する
fn task_letter(index: usize) -> String {
    if index < 26 {
        ((b'A' + index as u8) as char).to_string()
    } else {
        format!("[{index}]")
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Enumerate {
            periods,
            format,
            count,
        } => {
            let instance = PinwheelInstance::new(periods);
            match format {
                Format::Tasks => {
                    let schedules = enumerate_labeled(&instance);
                    if count {
                        println!("{}", schedules.len());
                        return;
                    }
                    println!("{:?}: {} schedule(s)", instance.periods(), schedules.len());
                    if !schedules.is_empty() {
                        // 各タスク（A=0番目, B=1番目, …）に周期を対応づけた凡例
                        let legend = instance
                            .periods()
                            .iter()
                            .enumerate()
                            .map(|(i, period)| format!("{}={}", task_letter(i), period))
                            .collect::<Vec<_>>()
                            .join(", ");
                        println!("labels: {legend}");
                    }
                    for schedule in &schedules {
                        let line: String = schedule.iter().map(|&task| task_letter(task)).collect();
                        println!("  {line}");
                    }
                }
                Format::Periods => {
                    let schedules = enumerate_schedules(&instance);
                    if count {
                        println!("{}", schedules.len());
                        return;
                    }
                    println!("{:?}: {} schedule(s)", instance.periods(), schedules.len());
                    for schedule in &schedules {
                        let line = schedule
                            .as_slice()
                            .iter()
                            .map(|period| period.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        println!("  {line}");
                    }
                }
            }
        }
        Command::Density { periods } => {
            let instance = PinwheelInstance::new(periods);
            let density = instance.density();
            let approx = *density.numer() as f64 / *density.denom() as f64;
            println!("{:?}: D = {density} ({approx:.4})", instance.periods());
        }
    }
}
