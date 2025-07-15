# Web Visualization Product Definition Document (PDD)

## Visualization Requirements

- The visualization should make effective use of the available screen space, automatically zooming to the area of interest.
- Each distinct region should be clearly demarcated with a border.
- Organisms should be clearly visible on top of the regions.

## 1. Overview

This document outlines the requirements for a web-based visualization of the `hill_descent` genetic algorithm simulation. The goal is to provide a real-time graphical representation of the world, its regions, and the organisms within it, running in a standard web browser. The visualization will be built using HTML, CSS, and JavaScript, with the D3.js library for rendering SVG elements.

The simulation itself will run as a WebAssembly (WASM) module, communicating with the JavaScript front-end to provide the necessary data for rendering at each step (round) of the simulation.

## 2. Core Requirements

### 2.1. Simulation Context

*   The simulation will be configured to use a fitness function with two input parameters and one output value. This corresponds to a 2-dimensional world where the "height" or "fitness" is the output.
*   The visualization updates after each round. Users can trigger a single round or run rounds continuously.
*   In continuous mode, a minimal delay (e.g., 10ms) is used between rounds to ensure a smooth animation.

### 2.2. Visualization Canvas

*   An SVG element will serve as the main canvas for the visualization.
*   The SVG canvas size is responsive. The view within the SVG (`viewBox`) will automatically pan and zoom to tightly frame the bounding box of all living organisms, ensuring the active area is always in focus.

### 2.3. World and Region Rendering

*   The world is a 2D plane defined by the global bounds of the two non-system parameters.
*   The background of the world will be a neutral color.
*   **Regions:**
    *   Each defined region will be rendered as an SVG `<rect>` (square/rectangle) on the world plane.
    *   The position and size of the rectangle will correspond to the region's boundaries.
    *   The fill color of the rectangle will represent the region's current minimum known height (score), mapped to a color scale.
    *   Region rectangles are rendered with 50% opacity to ensure that organisms on top are clearly visible.
*   **Color Scale:**
    *   A continuous color scale from blue (representing the global minimum score observed) through green to red (global maximum score observed) will be used.
    *   This scale will be used to color the region rectangles.
*   **Interactivity:**
    *   When a user hovers the mouse over a region rectangle, a tooltip or pop-up should appear, displaying the region's boundaries (e.g., "x: [0.5, 1.0], y: [0.0, 0.5]").

### 2.3. Legend / Metadata Display

*   A legend shall be rendered below the world grid showing:
    *   The current simulation round number (starting at 0 and incrementing each time the user presses **Run Round**).
    *   The current world bounds as reported by the backend, formatted as `x [min, max], y [min, max]`.
    *   The legend must update every time the visualization is refreshed.

### 2.4. Organism Rendering

*   **Representation:** Each organism will be rendered as an SVG `<circle>`.
*   **Color:** The fill color of the circle represents the organism's age, mapped to a continuous blue-green-red color scale (blue for young, red for old).
*   **Position:** The organism's position will be determined by its two non-system parameter values.
*   **Interactivity:**
    *   When a user hovers the mouse over an organism's circle, a tooltip or pop-up should appear, displaying its non-system parameters and its current age.

## 3. Technical Implementation Details

### 3.1. Technology Stack

*   **Frontend:** HTML, CSS, JavaScript
*   **JS Libraries:** D3.js (for SVG manipulation, data binding, and scales)
*   **Backend:** Rust compiled to WebAssembly (WASM)

### 3.2. Data Interface (WASM to JS)

The WASM module must expose a function that can be called from JavaScript to retrieve the state of the world after each round. This function should return a data structure (e.g., a JSON string or a JavaScript object) with the following information:

```json
{
  "world_bounds": {
    "x": [0.0, 100.0], // Min/Max for parameter 1
    "y": [0.0, 100.0]  // Min/Max for parameter 2
  },
  "score_range": {
    "min": -50.0,       // Global minimum score seen so far
    "max": 150.0        // Global maximum score seen so far
  },
  "regions": [
    {
      "id": 0,
      "bounds": {
        "x": [0.0, 10.0],
        "y": [0.0, 10.0]
      },
      "min_score": 12.5
    }
    // ... other regions
  ],
  "organisms": [
    {
      "id": 0,
      "params": {
        "x": 5.2,
        "y": 8.1
      },
      "age": 15
    }
    // ... other organisms
  ]
}
```

### 3.3. Control Flow

1.  **Initialization:** The JavaScript code initializes the D3.js visualization, sets up the SVG canvas, and defines the scales based on initial data from the WASM module.
2.  **Simulation Loop (JS):**
    a. Call the WASM function to run the next simulation round.
    b. Call the WASM function to get the updated world state.
    c. Use D3.js to update the visualization (regions and organisms) based on the new data. This involves a data join (`.data()`, `.join()`, `.enter()`, `.exit()`) to correctly add, update, and remove SVG elements.
    d. Wait for 1 second.
    e. Repeat.

## 4. Potential Enhancements (Future Work)

*   A continuous heatmap for the entire world background.
*   Controls to pause, resume, and restart the simulation.
*   Input fields to change simulation parameters on the fly.
*   Charts to show population size and average fitness over time.
