use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Result, middleware::Logger, web};
use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, sync::Mutex};

// Test function
#[derive(Debug, Clone)]
struct Himmelblau;

impl SingleValuedFunction for Himmelblau {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let term1 = (x.powi(2) + y - 11.0).powi(2);
        let term2 = (x + y.powi(2) - 7.0).powi(2);
        term1 + term2
    }
}

#[derive(Deserialize)]
struct StartRequest {
    population_size: Option<usize>,
    elite_size: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
struct StateResponse {
    epoch: usize,
    best_score: f64,
    world_state: String,
    at_resolution_limit: bool,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Configuration for recreating worlds
#[derive(Debug, Clone)]
struct Config {
    population_size: usize,
    elite_size: usize,
    param_ranges: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Default)]
struct AppState {
    current_state: Option<StateResponse>,
    config: Option<Config>,
}

// Actix Web handler functions
async fn start_handler(
    app_state: web::Data<Mutex<AppState>>,
    req: web::Json<StartRequest>,
) -> Result<HttpResponse> {
    let population_size = req.population_size.unwrap_or(100);
    let elite_size = req.elite_size.unwrap_or(10);
    let param_ranges = vec![(-25000000.0, -5000000.0), (-25000000.0, -5000000.0)];

    let config = Config {
        population_size,
        elite_size,
        param_ranges: param_ranges.clone(),
    };

    // Create initial world
    let param_range: Vec<RangeInclusive<f64>> = param_ranges
        .into_iter()
        .map(|(min, max)| RangeInclusive::new(min, max))
        .collect();

    let global_constants = GlobalConstants::new(population_size, elite_size);
    let world = setup_world(&param_range, global_constants, Box::new(Himmelblau));

    let response_data = StateResponse {
        epoch: 0,
        best_score: world.get_best_score(),
        // Return the web-shaped JSON for the frontend visualization
        world_state: world.get_state_for_web(),
        at_resolution_limit: false,
    };

    // Update state
    {
        let mut state = app_state.lock().unwrap();
        state.config = Some(config);
        state.current_state = Some(response_data.clone());
        // We do not persist the World here to keep server types Send/Sync-free
    }

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(response_data),
        error: None,
    }))
}

async fn step_handler(app_state: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let (config, current_epoch) = {
        let state = app_state.lock().unwrap();
        match (&state.config, &state.current_state) {
            (Some(config), Some(current_state)) => (config.clone(), current_state.epoch),
            _ => {
                return Ok(HttpResponse::BadRequest().json(ApiResponse {
                    success: false,
                    data: None::<()>,
                    error: Some("No optimization session started".to_string()),
                }));
            }
        }
    };

    // Recreate world and run to new epoch
    let param_range: Vec<RangeInclusive<f64>> = config
        .param_ranges
        .into_iter()
        .map(|(min, max)| RangeInclusive::new(min, max))
        .collect();

    let global_constants = GlobalConstants::new(config.population_size, config.elite_size);
    let mut world = setup_world(&param_range, global_constants, Box::new(Himmelblau));

    // Run to the new epoch
    let mut at_resolution_limit = false;
    for _ in 0..=current_epoch {
        at_resolution_limit = world.training_run(&[], &[]);
    }

    let response_data = StateResponse {
        epoch: current_epoch + 1,
        best_score: world.get_best_score(),
        // Return the web-shaped JSON for the frontend visualization
        world_state: world.get_state_for_web(),
        at_resolution_limit,
    };

    // Update state
    {
        let mut state = app_state.lock().unwrap();
        state.current_state = Some(response_data.clone());
    }

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(response_data),
        error: None,
    }))
}

async fn state_handler(app_state: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let state = app_state.lock().unwrap();

    if let Some(ref current_state) = state.current_state {
        Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(current_state.clone()),
            error: None,
        }))
    } else {
        Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<()>,
            error: Some("No optimization session started".to_string()),
        }))
    }
}

async fn reset_handler(app_state: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let mut state = app_state.lock().unwrap();
    state.current_state = None;
    state.config = None;

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let app_state = web::Data::new(Mutex::new(AppState::default()));

    println!("Server running on http://127.0.0.1:3000");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/api/start", web::post().to(start_handler))
            .route("/api/step", web::post().to(step_handler))
            .route("/api/state", web::get().to(state_handler))
            .route("/api/reset", web::post().to(reset_handler))
            .service({
                // Serve static files from the server crate's web/ directory
                let static_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
                Files::new("/", static_dir).index_file("index.html")
            })
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
