use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use console::{Term, style};
use dialoguer::{Input, Select};

#[derive(Parser)]
#[command(version, about = r#"
::::::.    :::. .::::::.:::::::::::::::.      :::      :::     
;;;`;;;;,  `;;;;;;`    `;;;;;;;;'''';;`;;     ;;;      ;;;     
[[[  [[[[[. '[['[==/[[[[,    [[    ,[[ '[[,   [[[      [[[     
$$$  $$$ "Y$c$$  '''    $    $$   c$$$cc$$$c  $$'      $$'     
888  888    Y88 88b    dP    88,   888   888,o88oo,.__o88oo,.__
MMM  MMM     YM  "YMmMY"     MMM   YMM   ""` """"YUMMM""""YUMMM
"#, long_about = None)]
struct Cli {
    /// minimal install
    #[arg(long)]
    minimal: bool,
}

fn clone_git_repo(repo_url: &str, repo_name: &str) -> io::Result<()> {
    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    let destination = format!("{}/{}", home_dir, repo_name);

    let status = Command::new("git")
        .args(["clone", repo_url, &destination])
        .status()?;

    if status.success() {
        println!(
            "{} - Repository cloned to {}",
            style("Success").green(),
            destination
        );
        Ok(())
    } else {
        eprintln!("Failed to clone the repository.");
        Err(io::Error::new(io::ErrorKind::Other, "Git clone failed"))
    }
}

fn copy_hardware_configuration() -> io::Result<()> {
    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    let destination = format!("{}/.nix/system", home_dir);

    let status = Command::new("cp")
        .args(["-f", "/etc/nixos/hardware-configuration.nix", &destination])
        .status()?;

    if status.success() {
        println!(
            "{} - Successfully copied hardware-configuration to {}",
            style("Success").green(),
            destination
        );
        Ok(())
    } else {
        eprintln!("Failed to clone the repository.");
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Hardware-configuration copy failed",
        ))
    }
}

fn run_sync() -> io::Result<()> {
    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    let path = format!("{}/Scripts/sync/target/release/sync", home_dir);

    let status = Command::new(path).arg("--skip-git").status()?;

    if status.success() {
        println!(
            "{} - Successfully ran system sync.",
            style("Success").green(),
        );
        Ok(())
    } else {
        eprintln!("Failed to run system sync.");
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Hardware-configuration copy failed",
        ))
    }
}

fn write_system_settings(gpu_driver: &str, hostname: &str) -> std::io::Result<()> {
    let ollama_acceleration = if gpu_driver == "NVIDIA" {
        "cuda"
    } else {
        "rocm"
    };

    let settings_nix_content = format!(
        r#"{{ config, pkgs, ... }}:

{{
  imports = [
    ./modules/boot.nix
    ./modules/gpu.nix
    ./modules/hyprland.nix
    ./modules/locale.nix
    ./modules/network.nix
    ./modules/ollama.nix
    ./modules/sound.nix
    ./modules/stylix.nix
    ./modules/zsh.nix
  ];

  bootSettings = {{
    enableAutoLogin = true;
    enableSystemdBoot = true;
  }};

  gpuSettings = {{
    enable = true;
    driver = "{}";
  }};

  hyprlandSettings = {{
    enable = true;
  }};

  localeSettings = {{
    enable = true;
    timeZone = "America/New_York";
    defaultLocale = "en_US.UTF-8";
  }};

  networkSettings = {{
    enable = true;
    hostName = "{}";
    enableNetworkManager = true;
  }};

  ollamaSettings = {{
    enable = true;
    acceleration = "{}";
  }};

  soundSettings = {{
    enable = true;
    enable32BitAlsa = true;
  }};

  stylixSettings = {{
    enable = true;
    wallpaper = ../wallpapers/sakura-wall.png;
    polarity = "dark";
  }};

  zshSettings = {{
    enable = true;
  }};
}}"#,
        gpu_driver, hostname, ollama_acceleration
    );

    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    let settings_path = format!("{}/.nix/system/settings.nix", home_dir);

    let mut file = File::create(settings_path)?;
    file.write_all(settings_nix_content.as_bytes())?;

    println!(
        "{} - System settings.nix has been genereated successfully.",
        style("Success").green()
    );
    Ok(())
}

fn write_user_settings(git_username: &str, git_email: &str) -> std::io::Result<()> {
    let user_settings_contents = format!(
        r#"{{config, lib, pkgs, ... }}:
    {{
        imports = [
            # firefox
            ./modules/firefox.nix
            # git
            ./modules/git.nix
            # helix
            ./modules/helix.nix
            # hyprland
            ./modules/hyprland.nix
            # kitty
            ./modules/kitty.nix
            # stylix
            ./modules/stylix.nix
            # zellij
            ./modules/zellij.nix
            # zsh
            ./modules/zsh/zsh.nix
        ];

        # firefox
        firefoxSettings = {{
            enable = true;
        }};

        # git
        gitSettings = {{
            enable = true;
            userName = "{}";
            userEmail = "{}";
        }};

        # helix
        helixSettings = {{
            enable = true;
            settings = {{
                editor.cursor-shape = {{
                    normal = "block";
                    insert = "bar";
                    select = "underline";
                }};
                editor = {{
                    line-number = "relative";
                    lsp.display-messages = true;
                }};
            }};
            languages = {{
               language-server.rust-analyzer.config.check = {{
                   command = "clippy";
               }};
            }};
            packages = [ pkgs.marksman pkgs.unstable.lldb_19 pkgs.rust-analyzer];
        }};

        # hyprland
        hyprlandSettings = {{
            enable = true;
            settings = {{
                exec-once = ["waybar"];
                env = [
                    "XDG_CURRENT_DESKTOP,Hyprland"
                    "XDG_SESSION_TYPE,wayland"
                    "XDG_SESSION_DESKTOP,Hyprland"
                ];
                monitor = ",highrr,auto,1";

                general = {{
                    gaps_in = 5;
                    gaps_out = 20;
                    border_size = 3;
                    allow_tearing = false;
                    layout = "dwindle";
                }};

                decoration = {{
                    rounding = 5;
                    inactive_opacity = 0.70;
                    blur = {{
                        enabled = true;
                        size = 3;
                        passes = 3;
                        vibrancy = 0.1696;
                    }};
                }};

                animations = {{
                    enabled = true;
                    bezier = "myBezier, 0.05, 0.9, 0.1, 1.05";
                }};

                dwindle = {{
                    pseudotile = true;
                    preserve_split = true;
                }};

                "$mod" = "SUPER";

                bind = [
                    "$mod, Q, exec, kitty"
                    "$mod, R, exec, tofi-drun"
                    "$mod, C, killactive"
                    "$mod, M, exit"

                    "$mod, P, pseudo"
                    "$mod, V, togglefloating"

                    "$mod, H, movefocus, l"
                    "$mod, J, movefocus, d"
                    "$mod, K, movefocus, u"
                    "$mod, L, movefocus, r"

                    "$mod SHIFT, H, resizeactive, -20 0"
                    "$mod SHIFT, J, resizeactive, 0 20"
                    "$mod SHIFT, K, resizeactive, 0 -20"
                    "$mod SHIFT, L, resizeactive, 20 0"

                    "$mod CTRL, H, swapwindow, l"
                    "$mod CTRL, J, swapwindow, d"
                    "$mod CTRL, K, swapwindow, u"
                    "$mod CTRL, L, swapwindow, r"

                ] ++ (
                    builtins.concatLists (builtins.genList(i:
                        let ws = i + 1;
                        in [
                            "$mod, code:1${{toString i}}, workspace, ${{toString ws}}"
                            "$mod SHIFT, code:1${{toString i}}, movetoworkspace, ${{toString ws}}"
                    ]) 9)
                );

                binde = [
                    "$mod CTRL, H, moveactive, -50 0"
                    "$mod CTRL, J, moveactive, 0 50"
                    "$mod CTRL, K, moveactive, 0 -50"
                    "$mod CTRL, L, moveactive, 50 0"
                ];

                windowrulev2 = [
                  "suppressevent maximize, class:.*"
                ];
            }};
        }};

        # kitty
        kittySettings = {{
            enable = true;
            settings = {{
                confirm_os_window_close = 0;
                enable_audio_bell = false;
                window_padding_width = 10;
            }};
        }};

        # stylix
        stylixSettings = {{
           enable = true;
           wallpaper = ../wallpapers/sakura-wall.png;
           polarity = "dark";
           opacity = {{
               desktop = 0.4;
           }};
        }};

        # zellij
        zellijSettings = {{
            enable = true;
        }};

        # zsh 
        zshSettings = {{
           enable = true;
           aliases = {{
               shutdown = "shutdown now";
               ls = "eza --icons -a --group-directories-first";
               tree = "eza --color=auto --tree";
               dev = "nix develop ~/.system/shell";
           }};
           plugins = [
               "zsh-users/zsh-autosuggestions"  
               "zsh-users/zsh-syntax-highlighting"
               "romkatv/powerlevel10k"
               "jeffreytse/zsh-vi-mode"
           ];
           zshrc = ''
            source ~/.nix/user/modules/zsh/p10k.zsh
           '';
           extraPackages = [ pkgs.noto-fonts-cjk-sans pkgs.eza pkgs.cmatrix pkgs.bottom pkgs.ttyper pkgs.neofetch pkgs.cloc pkgs.pom pkgs.cava ];
        }};
    }}
    "#,
        git_username, git_email
    );

    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    let settings_path = format!("{}/.nix/user/settings.nix", home_dir);

    let mut file = File::create(settings_path)?;
    file.write_all(user_settings_contents.as_bytes())?;

    println!(
        "{} - User settings.nix has been genereated successfully.",
        style("Success").green()
    );

    Ok(())
}

fn main() {
    // Parse cli args
    let cli = Cli::parse();

    if cli.minimal {
        // Setup the terminal
        let term = Term::stdout();
        let _ = term.clear_screen();

        // Print banner
        let banner = r#"
::::::.    :::. .::::::.:::::::::::::::.      :::      :::     
;;;`;;;;,  `;;;;;;`    `;;;;;;;;'''';;`;;     ;;;      ;;;     
[[[  [[[[[. '[['[==/[[[[,    [[    ,[[ '[[,   [[[      [[[     
$$$  $$$ "Y$c$$  '''    $    $$   c$$$cc$$$c  $$'      $$'     
888  888    Y88 88b    dP    88,   888   888,o88oo,.__o88oo,.__
MMM  MMM     YM  "YMmMY"     MMM   YMM   ""` """"YUMMM""""YUMMM
"#;
        println!("{}", banner);

        // Git repo vars
        let repo_url = "https://github.com/rottedfm/nix.git";
        let repo_name = ".nix";

        // Clone git repo
        println!("{} - Cloning git repo...", style("Info").cyan());
        match clone_git_repo(repo_url, &repo_name) {
            Ok(_) => println!("{} - Cloned git repo.", style("Success").green()),
            Err(e) => eprintln!("{} - {}.", style("Error").red(), e),
        }

        // wait time for clearing
        sleep(Duration::from_millis(500));
        let _ = term.clear_screen();
        println!("{}", banner);

        println!(
            "{} - Copying hardware-configuration to .nix/system.",
            style("Info").cyan()
        );

        if let Err(e) = copy_hardware_configuration() {
            eprintln!("Error copying hardware-configuration {}", e);
        }

        // wait time for clearing
        sleep(Duration::from_secs(1));
        let _ = term.clear_screen();
        println!("{}", banner);

        let gpu_driver_options = vec!["AMD", "NVIDIA"];

        let gpu_driver_index = Select::new()
            .with_prompt("What type of GPU is in your device?")
            .items(&gpu_driver_options)
            .default(0)
            .interact()
            .unwrap();

        let gpu_driver = gpu_driver_options[gpu_driver_index];

        // wait time for clearing
        sleep(Duration::from_millis(250));
        let _ = term.clear_screen();
        println!("{}", banner);

        let hostname: String = Input::new()
            .with_prompt("Please set your hostname")
            .interact_text()
            .unwrap();

        // wait time for clearing
        sleep(Duration::from_millis(250));
        let _ = term.clear_screen();
        println!("{}", banner);

        if let Err(e) = write_system_settings(&gpu_driver, &hostname) {
            eprintln!("Error writing settings.nix: {}", e);
        }

        // wait time for clearing
        sleep(Duration::from_millis(500));
        let _ = term.clear_screen();
        println!("{}", banner);

        let git_username: String = Input::new()
            .with_prompt("Please input your git username")
            .interact_text()
            .unwrap();

        // wait time for clearing
        sleep(Duration::from_millis(250));
        let _ = term.clear_screen();
        println!("{}", banner);

        let git_email: String = Input::new()
            .with_prompt("Please input your git email")
            .interact_text()
            .unwrap();

        // wait time for clearing
        sleep(Duration::from_millis(250));
        let _ = term.clear_screen();
        println!("{}", banner);

        if let Err(e) = write_user_settings(&git_username, &git_email) {
            eprintln!("Error writing settings.nix: {}", e);
        }

        // wait time for clearing
        sleep(Duration::from_millis(250));
        let _ = term.clear_screen();
        println!("{}", banner);

        let sync_repo_url = "https://github.com/rottedfm/sync.git";
        let sync_repo_name = "Scripts/sync";

        // Clone git repo
        println!("{} - Cloning git repo...", style("Info").cyan());
        match clone_git_repo(sync_repo_url, &sync_repo_name) {
            Ok(_) => println!("{} - Cloned git repo.", style("Success").green()),
            Err(e) => eprintln!("{} - {}.", style("Error").red(), e),
        }

        if let Err(e) = run_sync() {
            eprintln!("Error running sync: {}", e);
        }
    }
}
