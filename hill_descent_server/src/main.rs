use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Result, middleware::Logger, web};
use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
    world::world_function::WorldFunction,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::RangeInclusive, sync::Mutex};

/// Available optimization functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    Himmelblau,
    Rastrigin,
    Ackley,
    BukinN6,
}
/// Bukin N.6 function implementation (2D, narrow curved valley)
/// f(x, y) = 100 * sqrt(|y - 0.01x^2|) + 0.01 * |x + 10|
/// Global minimum at (-10, 1) with f = 0.0
#[derive(Debug, Clone)]
struct BukinN6;

impl SingleValuedFunction for BukinN6 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let term1 = 100.0 * (y - 0.01 * x * x).abs().sqrt();
        let term2 = 0.01 * (x + 10.0).abs();
        term1 + term2
    }
}

/// Himmelblau function implementation
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

/// Rastrigin function implementation (2D multimodal benchmark)
/// f(x, y) = 20 + (x^2 - 10 cos(2πx)) + (y^2 - 10 cos(2πy))
/// Global minimum at (0,0) with f = 0.0
#[derive(Debug, Clone)]
struct Rastrigin;

impl SingleValuedFunction for Rastrigin {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        let two_pi = 2.0 * std::f64::consts::PI;
        20.0 + (x * x - 10.0 * (two_pi * x).cos()) + (y * y - 10.0 * (two_pi * y).cos())
    }
}

/// Ackley function implementation (2D multimodal benchmark)
/// f(x, y) = -20 * exp(-0.2 * sqrt(0.5 * (x^2 + y^2))) - exp(0.5 * (cos(2πx) + cos(2πy))) + e + 20
/// Global minimum at (0,0) with f = 0.0
#[derive(Debug, Clone)]
struct Ackley;

impl SingleValuedFunction for Ackley {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        let two_pi = 2.0 * std::f64::consts::PI;
        let term1 = -20.0 * (-0.2 * (0.5 * (x * x + y * y)).sqrt()).exp();
        let term2 = -(0.5 * ((two_pi * x).cos() + (two_pi * y).cos())).exp();
        let e = std::f64::consts::E;

        term1 + term2 + e + 20.0
    }
}

/// Function metadata for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub description: String,
    pub param_ranges: Vec<(f64, f64)>,
    pub global_minimum: Option<(f64, f64)>,
}

/// Function registry to create function instances and metadata
pub struct FunctionRegistry {
    functions: HashMap<FunctionType, FunctionInfo>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        functions.insert(
            FunctionType::Himmelblau,
            FunctionInfo {
                name: "Himmelblau".to_string(),
                description: "Himmelblau's function - multimodal with 4 global minima".to_string(),
                param_ranges: vec![(-25000000.0, -5000000.0), (-25000000.0, -5000000.0)],
                global_minimum: Some((3.0, 2.0)), // One of the four minima
            },
        );

        functions.insert(
            FunctionType::Rastrigin,
            FunctionInfo {
                name: "Rastrigin".to_string(),
                description: "Rastrigin function - highly multimodal with many local minima"
                    .to_string(),
                param_ranges: vec![(-5.12, 5.12), (-5.12, 5.12)],
                global_minimum: Some((0.0, 0.0)),
            },
        );

        functions.insert(
            FunctionType::Ackley,
            FunctionInfo {
                name: "Ackley".to_string(),
                description: "Ackley function - complex multimodal function with exponential terms"
                    .to_string(),
                param_ranges: vec![(-5.0, 5.0), (-5.0, 5.0)],
                global_minimum: Some((0.0, 0.0)),
            },
        );

        functions.insert(
            FunctionType::BukinN6,
            FunctionInfo {
                name: "Bukin N.6".to_string(),
                description: "Bukin N.6 function - narrow curved valley, very challenging"
                    .to_string(),
                param_ranges: vec![(-15.0, -5.0), (-3.0, 3.0)],
                global_minimum: Some((-10.0, 1.0)),
            },
        );

        Self { functions }
    }

    pub fn get_function_info(&self, function_type: &FunctionType) -> Option<&FunctionInfo> {
        self.functions.get(function_type)
    }

    pub fn list_functions(&self) -> HashMap<FunctionType, FunctionInfo> {
        self.functions.clone()
    }

    pub fn create_function(&self, function_type: &FunctionType) -> Option<Box<dyn WorldFunction>> {
        match function_type {
            FunctionType::Himmelblau => Some(Box::new(Himmelblau)),
            FunctionType::Rastrigin => Some(Box::new(Rastrigin)),
            FunctionType::Ackley => Some(Box::new(Ackley)),
            FunctionType::BukinN6 => Some(Box::new(BukinN6)),
        }
    }
}

#[derive(Deserialize)]
struct StartRequest {
    population_size: Option<usize>,
    elite_size: Option<usize>,
    function_type: Option<FunctionType>,
}

#[derive(Serialize, Clone, Debug)]
struct StateResponse {
    epoch: usize,
    best_score: f64,
    world_state: String,
    at_resolution_limit: bool,
    function_type: FunctionType,
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
    function_type: FunctionType,
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
    let function_type = req
        .function_type
        .clone()
        .unwrap_or(FunctionType::Himmelblau);

    let registry = FunctionRegistry::new();
    let function_info = registry
        .get_function_info(&function_type)
        .ok_or_else(|| format!("Unknown function type: {:?}", function_type))
        .map_err(actix_web::error::ErrorBadRequest)?;

    let config = Config {
        population_size,
        elite_size,
        param_ranges: function_info.param_ranges.clone(),
        function_type: function_type.clone(),
    };

    // Create initial world
    let param_range: Vec<RangeInclusive<f64>> = function_info
        .param_ranges
        .iter()
        .map(|(min, max)| RangeInclusive::new(*min, *max))
        .collect();

    let function_impl = registry
        .create_function(&function_type)
        .ok_or("Failed to create function implementation")
        .map_err(|e| {
            eprintln!("Error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let global_constants = GlobalConstants::new(population_size, elite_size);
    let world = setup_world(&param_range, global_constants, function_impl);

    let response_data = StateResponse {
        epoch: 0,
        best_score: world.get_best_score(),
        // Return the web-shaped JSON for the frontend visualization
        world_state: world.get_state_for_web(),
        at_resolution_limit: false,
        function_type,
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

    let registry = FunctionRegistry::new();
    let function_impl = registry
        .create_function(&config.function_type)
        .ok_or("Failed to create function implementation")
        .map_err(|e| {
            eprintln!("Error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let global_constants = GlobalConstants::new(config.population_size, config.elite_size);
    let mut world = setup_world(&param_range, global_constants, function_impl);

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
        function_type: config.function_type,
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

async fn functions_handler() -> Result<HttpResponse> {
    let registry = FunctionRegistry::new();
    let functions = registry.list_functions();

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(functions),
        error: None,
    }))
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
            .route("/api/functions", web::get().to(functions_handler))
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
