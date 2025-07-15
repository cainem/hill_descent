import init, { WasmWorld } from "../pkg/hill_descent.js";

async function main() {
    // 1. Initialize WASM
    await init();
    const world = WasmWorld.new();
    let round = 0; // Track simulation rounds

    // 2. Setup D3.js visualization elements
    const margin = { top: 20, right: 20, bottom: 30, left: 40 };
    const width = 800 - margin.left - margin.right;
    const height = 800 - margin.top - margin.bottom;

    const svg = d3.select("#visualization").append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom)
      .append("g")
        .attr("transform", `translate(${margin.left},${margin.top})`);

    const tooltip = d3.select("#tooltip");

    // Legend div below the visualization
    const legend = d3.select("body")
        .append("div")
        .attr("id", "legend")
        .style("margin", "10px")
        .style("font-family", "sans-serif")
        .style("font-size", "14px");

    // Add a background rectangle for the entire world
    svg.append("rect")
        .attr("class", "world-background")
        .attr("x", 0)
        .attr("y", 0)
        .attr("width", width)
        .attr("height", height)
        .attr("fill", "#f0f0f0")
        .on("mouseover", (event, d) => {
            tooltip.transition().duration(200).style("opacity", .9);
            tooltip.html(`Unoccupied Territory`)
                .style("left", (event.pageX + 5) + "px")
                .style("top", (event.pageY - 28) + "px");
        })
        .on("mouseout", () => {
            tooltip.transition().duration(500).style("opacity", 0);
        });

    // Define scales
    const xScale = d3.scaleLinear()
        .domain([-500, 500])
        .range([0, width]);

    const yScale = d3.scaleLinear()
        .domain([-500, 500])
        .range([height, 0]);

    // 3. Define the rendering function
    function updateVisualization() {
        const state = JSON.parse(world.get_state_for_web());

        const ageColorScale = d3.scaleSequential(d3.interpolateRgbBasis(["blue", "green", "red"]))
            .domain([0, 1]); // Domain is now age percentage (0 to 1)



        // Calculate display (occupied) bounds based on populated regions; fallback to world bounds if none
        let displayBounds = {
            x: [...state.world_bounds.x],
            y: [...state.world_bounds.y]
        };
        // Determine bounds, prioritizing populated regions, then all regions, then organisms
        {
            // An 'active' region is one with carrying capacity OR organisms inside it.
            const activeRegions = (state.regions || []).filter(region => {
                if (region.carrying_capacity > 0) {
                    return true;
                }
                // Check if any organism is within this region's bounds
                return (state.organisms || []).some(organism =>
                    organism.params.x >= region.bounds.x[0] && organism.params.x <= region.bounds.x[1] &&
                    organism.params.y >= region.bounds.y[0] && organism.params.y <= region.bounds.y[1]
                );
            });

            if (activeRegions.length > 0) {
                // If we have active regions, calculate a bounding box to contain all of them.
                const allX = activeRegions.flatMap(r => r.bounds.x);
                const allY = activeRegions.flatMap(r => r.bounds.y);
                displayBounds.x[0] = d3.min(allX);
                displayBounds.x[1] = d3.max(allX);
                displayBounds.y[0] = d3.min(allY);
                displayBounds.y[1] = d3.max(allY);
            } else if (state.organisms && state.organisms.length > 0) {
                // Fallback for the rare case of organisms but no regions: bound the organisms.
                displayBounds.x[0] = d3.min(state.organisms, o => o.params.x);
                displayBounds.x[1] = d3.max(state.organisms, o => o.params.x);
                displayBounds.y[0] = d3.min(state.organisms, o => o.params.y);
                displayBounds.y[1] = d3.max(state.organisms, o => o.params.y);
            }
            // If there's nothing, we just keep the default world bounds.
            // If all else fails, displayBounds remains at world bounds as initialized
        }



        // Update legend showing both world & displayed bounds
        legend.html(`Round: ${round}` +
            `<br/>World bounds: x [${state.world_bounds.x[0].toFixed(2)}, ${state.world_bounds.x[1].toFixed(2)}], ` +
            `y [${state.world_bounds.y[0].toFixed(2)}, ${state.world_bounds.y[1].toFixed(2)}]` +
            `<br/>Displayed bounds: x [${displayBounds.x[0].toFixed(2)}, ${displayBounds.x[1].toFixed(2)}], ` +
            `y [${displayBounds.y[0].toFixed(2)}, ${displayBounds.y[1].toFixed(2)}]`);

        // Adjust bounds to maintain square aspect
        {
            const xSpan = displayBounds.x[1] - displayBounds.x[0];
            const ySpan = displayBounds.y[1] - displayBounds.y[0];
            const maxSpan = Math.max(xSpan, ySpan);
            const padSpanX = (maxSpan - xSpan) / 2;
            const padSpanY = (maxSpan - ySpan) / 2;
            if (padSpanX > 0) {
                displayBounds.x[0] -= padSpanX;
                displayBounds.x[1] += padSpanX;
            }
            if (padSpanY > 0) {
                displayBounds.y[0] -= padSpanY;
                displayBounds.y[1] += padSpanY;
            }

            // final padding 12%
            const pad = (min, max) => {
                const span = max - min;
                const padVal = span * 0.12 || 1e-3;
                return [min - padVal, max + padVal];
            };
            [displayBounds.x[0], displayBounds.x[1]] = pad(displayBounds.x[0], displayBounds.x[1]);
            [displayBounds.y[0], displayBounds.y[1]] = pad(displayBounds.y[0], displayBounds.y[1]);
        }



        // Update scales to display bounds (auto-zoom)
        xScale.domain(displayBounds.x);
        yScale.domain(displayBounds.y);


        // Assertion: Check if every organism is within a provided region.
        state.organisms.forEach(organism => {
            const is_in_a_region = state.regions.some(region => {
                return organism.params.x >= region.bounds.x[0] &&
                       organism.params.x <= region.bounds.x[1] &&
                       organism.params.y >= region.bounds.y[0] &&
                       organism.params.y <= region.bounds.y[1];
            });

            console.assert(is_in_a_region, `Organism found outside of any region!`, { organism });
        });

        const colorScale = d3.scaleSequential(d3.interpolateRdYlBu)
            .domain([state.score_range.max, state.score_range.min]); // Inverted to make blue=low, red=high

        // Render Regions
        const regions = svg.selectAll(".region")
            .data(state.regions, d => `${d.bounds.x[0]}-${d.bounds.y[0]}`);

        regions.enter().append("rect")
            .attr("class", "region")
            .merge(regions)
            .attr("x", d => xScale(d.bounds.x[0]))
            .attr("y", d => yScale(d.bounds.y[1]))
            .attr('width', d => xScale(d.bounds.x[1]) - xScale(d.bounds.x[0]))
            .attr('height', d => yScale(d.bounds.y[0]) - yScale(d.bounds.y[1]))
            .style('fill', d => d.min_score === null ? "rgba(200, 200, 200, 0.5)" : colorScale(d.min_score))
            .style('stroke', 'black') // Add black border to regions
            .style('stroke-width', 1)
            .on("mouseover", (event, d) => {
                tooltip.transition().duration(200).style("opacity", .9);
                tooltip.html(`Region Bounds:<br/>  x: [${d.bounds.x[0].toFixed(2)}, ${d.bounds.x[1].toFixed(2)}]<br/>  y: [${d.bounds.y[0].toFixed(2)}, ${d.bounds.y[1].toFixed(2)}]<br/>Min Score: ${d.min_score ? d.min_score.toExponential(2) : 'N/A'}`)
                    .style("left", (event.pageX + 5) + "px")
                    .style("top", (event.pageY - 28) + "px");
            })
            .on("mouseout", () => {
                tooltip.transition().duration(500).style("opacity", 0);
            });

        regions.exit().remove();

        // Draw carrying capacity text
        const regionTexts = svg.selectAll('.region-text')
            .data(state.regions, d => d.bounds.x[0] + "," + d.bounds.y[0]);

        regionTexts.enter()
            .append('text')
            .attr('class', 'region-text')
            .merge(regionTexts)
            .attr('x', d => xScale(d.bounds.x[0]) + (xScale(d.bounds.x[1]) - xScale(d.bounds.x[0])) / 2)
            .attr('y', d => yScale(d.bounds.y[1]) + (yScale(d.bounds.y[0]) - yScale(d.bounds.y[1])) / 2)
            .attr('dy', '0.35em') // Vertically center
            .style('text-anchor', 'middle')
            .style('fill-opacity', 1.0)
            .style('font-size', '12px')
            .style('fill', 'black')
            .text(d => d.carrying_capacity);

        regionTexts.exit().remove();

        // Render Organisms
        const organisms = svg.selectAll(".organism")
            .data(state.organisms);

        organisms.enter().append("circle")
            .attr("class", "organism")
            .attr("r", 3)
            .merge(organisms)
            .attr("fill", d => ageColorScale(d.age / d.max_age))
            .attr("cx", d => xScale(d.params.x))
            .attr("cy", d => yScale(d.params.y))
            .raise() // ensure circles are on top of region rectangles
            .on("mouseover", (event, d) => {
                tooltip.transition().duration(200).style("opacity", .9);
                tooltip.html(`Organism:<br/>  x: ${d.params.x.toFixed(2)}<br/>  y: ${d.params.y.toFixed(2)}<br/>Age: ${d.age}`)
                    .style("left", (event.pageX + 5) + "px")
                    .style("top", (event.pageY - 28) + "px");
            })
            .on("mouseout", () => {
                tooltip.transition().duration(500).style("opacity", 0);
            });

        organisms.exit().remove();
    }

    // 4. Start the simulation loop
    function simulationLoop() {
        round += 1;
        world.training_run();
        updateVisualization();
    }

    // Initial render
    updateVisualization(); 

    // Add a button to manually advance the simulation
    const runButton = d3.select("body")
        .append("button")
        .attr("id", "run-button")
        .text("Run Round")
        .on("click", simulationLoop);

}

main();
