#[cfg(feature = "Chess")]
extern crate chess;

use chess::app::{App, AppResult};
use chess::constants::{home_dir, DisplayMode};
use chess::event::{Event, EventHandler};
use chess::game_logic::opponent::wait_for_game_start;
use chess::handler::{handle_key_events, handle_mouse_events};
use chess::logging;
use chess::ui::tui::Tui;
use clap::Parser;

use log::LevelFilter;
use std::fs::{self, File};
use std::io::Write;
use std::panic;
use std::path::Path;
use toml::Value;
use chess::game_logic::game::GameState;

#[derive(Parser,Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "")]
    engine_path: String,
}


fn main() -> AppResult<()> {
    ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::event::EnableMouseCapture
    )?;

    let args = Args::parse();

    let home_dir = home_dir()?;

    let folder_path = home_dir.join(".config/chess");
    let config_path = home_dir.join(".config/chess/config.toml");


    config_create(&args, &folder_path, &config_path)?;

    let mut app  = App::default();

    if let Ok(content) = fs::read_to_string(config_path) {
      if content.trim().is_empty() {
          app.chess_engine_path = None;
      } else {
          let config = content.parse::<Value>().unwrap();
          if let Some(engine_path) = config.get("engine_path"){
              app.chess_engine_path = Some(engine_path.as_str().unwrap().to_string());
          }
          if let Some(display_mode) =config.get("display_mode") {
              app.game.ui.display_mode = match display_mode.as_str() {
                  _=> DisplayMode::DEFAULT,
              };
          }
          if let Some(log_level) = config.get("log_level") {
              app.log_level = log_level
                  .as_str()
                  .and_then(|s| s.parse().ok())
                  .unwrap_or(LevelFilter::Off);
          }
      }
    } else {
        println!("Error reading the file or the file does not the exist");
    }

    if let Err(e) = logging::setup_logging(&folder_path, &app.log_level) {
        eprint!("Failed to initialize logging: {}", e);
    }

    let terminal = ratatui::try_init()?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);

    let default_panic = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        ratatui::crossterm::execute!(
            std::io::stdout(),
            ratatui::crossterm::event::DisableMouseCapture
        )
            .unwrap();
        default_panic(info);
    }));


    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(mouse_event) => handle_mouse_events(mouse_event, &mut app)?,
            Event::Resize(_, _) => {}
        }

        if app.game.bot.is_some() && app.game.bot.as_ref().is_some_and(|bot| bot.bot_will_move) {
            app.game.execute_bot_move();
            app.game.switch_player_turn();
            if let Some(bot) = app.game.bot.as_mut() {
                bot.bot_will_move = false;
            }

            if app.game.game_board.is_checkmate(app.game.player_turn) {
                app.game.game_state = GameState::Checkmate;
            } else if app.game.game_board.is_draw(app.game.player_turn) {
                app.game.game_state = GameState::Draw;
            }
            tui.draw(&mut app)?;
        }

        if app.game.opponent.is_some() && app.game.opponent.as_ref().is_some_and(|opponent| !opponent.game_started) {
                let opponent = app.game.opponent.as_mut().unwrap();
                wait_for_game_start(opponent.stream.as_ref().unwrap());
                opponent.game_started = true;
                app.current_popup = None;
            }

            if app.game.opponent.is_some() && app.game.opponent.as_ref().is_some_and(|opponent| opponent.opponent_will_move) {
                    tui.draw(&mut app)?;

                    if !app.game.game_board.is_checkmate(app.game.player_turn) && !app.game.game_board.is_draw(app.game.player_turn) {
                        app.game.execute_opponent_move();
                        app.game.switch_player_turn();
                    }

                    if app.game.game_board.is_checkmate(app.game.player_turn) {
                        app.game.game_state = GameState::Checkmate;
                    } else if app.game.game_board.is_draw(app.game.player_turn) {
                        app.game.game_state = GameState::Draw;
                    }
                    tui.draw(&mut app)?;
                }
            }

            ratatui::try_restore()?;

            ratatui::crossterm::execute! (
                std::io::stdout(),
                ratatui::crossterm::event::DisableMouseCapture
            )?;
            Ok(())
        }


fn config_create(args: &Args, folder_path:&Path, config_path: &Path) -> AppResult<()> {
    fs::create_dir_all(folder_path)?;

    if !config_path.exists() {
        File::create(config_path)?;
    }


    let mut config = match fs::read_to_string(config_path) {
        Ok(content) => content
            .parse::<Value>()
            .unwrap_or_else(|_| Value::Table(Default::default())),
        Err(_) => Value::Table(Default::default()),
    };

    if let Some(table) = config.as_table_mut() {
        if args.engine_path.is_empty() {
            table
                .entry("engine_path".to_string())
                .or_insert(Value::String(String::new()));
        } else {
            table.insert(
                "engine_path".to_string(),
                Value::String(args.engine_path.clone()),
            );
        }
        table
            .entry("display_mode".to_string())
            .or_insert(Value::String("DEFAULT".to_string()));

        table
            .entry("log_level".to_string())
            .or_insert(Value::String(LevelFilter::Off.to_string()));
    }


    let mut file = File::create(config_path)?;
    file.write_all(config.to_string().as_bytes())?;

    Ok(())

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use toml::Value;

    #[test]
    fn tets_config_create() {
        let args = Args {
            engine_path:  "test_engine_path".to_string(),
        };

        let home_dir = home_dir().expect("failed to get home directory");
        let folder_path = home_dir.join(".test/chess");
        let config_path = home_dir.join("test/chess/config.toml");


        let result = config_create(&args, &folder_path, &config_path);

        assert!(result.is_ok());
        assert!(config_path.exists());


        let content = fs::read_to_string(config_path).unwrap();
        let config: Value = content.parse().unwrap();
        let table = config.as_table().unwrap();


        assert_eq!(
            table.get("engine_path").unwrap().as_str().unwrap(),
            "test_engine_path"
        );

        assert_eq!(
            table.get("display_mode").unwrap().as_str().unwrap(),
            "DEFAULT"
        );

        let removed = fs::remove_dir_all(home_dir.join(".test"));
        assert!(removed.is_ok());
    }
}