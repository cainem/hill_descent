# Web Visualization Product Definition Document (PDD)

## Visualization Requirements

- The visualization should make effective use of the available screen space, automatically zooming to the area of interest.
- Each distinct region should be clearly demarcated with a border.
- Organisms should be clearly visible on top of the regions.

## 1. Overview

This document outlines the requirements for a web-based visualization of the `hill_descent` genetic algorithm simulation. The goal is to provide a real-time graphical representation of the world, its regions, and the organisms within it, running in a standard web browser. The visualization is built using HTML, CSS, and JavaScript, with the D3.js library for rendering SVG elements.

The simulation runs on a native Rust server (Actix Web). The frontend communicates with the server via JSON REST API endpoints to retrieve state and advance the simulation. There is no longer any WebAssembly component.

## 2. Core Requirements

### 2.1. Simulation Context

*   The simulation uses a fitness function with two input parameters and one output value. This corresponds to a 2-dimensional world where the "height" or "fitness" is the output.
*   The visualization updates after each round. Users can trigger a single round or run rounds continuously.
*   In continuous mode, requests are serialized (no overlapping calls). A 500ms delay is used between rounds to ensure stable visualization and pacing.

### 2.2. Visualization Canvas

*   An SVG element serves as the main canvas for the visualization.
*   The SVG canvas size is responsive. The view within the SVG (`viewBox`) automatically pans and zooms to tightly frame the bounding box of all living organisms, ensuring the active area is always in focus.
*   Explicit SVG layer groups control z-order: `gRegions` (regions), `gOrganisms` (organisms), `gOverlay` (tooltips/overlays), `gCorners` (corner labels). Organisms must render above regions. Overlay and corner groups must not intercept pointer events (`pointer-events: none`).
*   Corner coordinate labels are rendered within the SVG at the four corners to display current visible world bounds.

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
    *   When a user hovers the mouse over a region rectangle, a tooltip appears displaying the region boundaries and value summary.

### 2.3. Metadata Display

*   Simplified UI with no separate legend, no bottom charts, and no world state text box.
*   The current visible world bounds are shown via the four in-canvas corner labels.
*   Tooltips provide detailed data on demand (regions and organisms).

### 2.4. Organism Rendering

*   **Representation:** Each organism is rendered as an SVG `<circle>` (radius ~4) with a thin white stroke to improve contrast over colored regions.
*   **Color:** The fill color represents the organism's age, mapped to a continuous blue-green-red color scale (blue for young, red for old).
*   **Position:** The organism's position is determined by its two non-system parameter values.
*   **Interactivity:**
    *   Hovering a circle shows a tooltip displaying its non-system parameters and current age.

## 3. Technical Implementation Details

### 3.1. Technology Stack

*   **Frontend:** HTML, CSS, JavaScript
*   **JS Libraries:** D3.js (for SVG manipulation, data binding, and scales)
*   **Backend:** Rust (Actix Web) server exposing JSON REST endpoints

### 3.2. Data Interface (Server REST API)

The server exposes the following endpoints:

*   `POST /api/start` → Starts a session. Returns `{ success, data: { epoch, best_score, world_state, at_resolution_limit } }`.
*   `POST /api/step` → Advances one round. Returns the same shape.
*   `GET  /api/state` → Returns the current state if started.
*   `POST /api/reset` → Clears the session.

`world_state` is a JSON string representing the visualization payload consumed by the frontend (parsed by JS). Its structure is:

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

1.  **Initialization:** JavaScript initializes the D3.js visualization, sets up the SVG canvas and layer groups, fetches initial state via `POST /api/start`, parses `world_state`, sets scales, and renders.
2.  **Simulation Loop (JS):**
    a. Issue `POST /api/step` and await completion (guard to prevent overlapping requests).
    b. Parse `world_state` from the response.
    c. Update the visualization (regions, organisms, overlays, corner labels) using D3 data joins.
    d. Wait 500ms.
    e. Repeat while auto-run is enabled.

## 4. Potential Enhancements (Future Work)

*   A continuous heatmap for the entire world background.
*   Controls to pause, resume, and restart the simulation.
*   Input fields to change simulation parameters on the fly.
*   Charts to show population size and average fitness over time.
