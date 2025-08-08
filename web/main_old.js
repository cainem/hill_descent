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

    // Corner coordinate labels will be added directly to the SVG

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



        // Add corner coordinate labels directly on the visualization
        // Remove any existing corner labels first
        svg.selectAll(".corner-label").remove();
        
        // Add coordinate labels at the four corners
        const cornerLabels = [
            { x: 0, y: 0, anchor: "start", baseline: "hanging", coords: `(${displayBounds.x[0].toFixed(1)}, ${displayBounds.y[1].toFixed(1)})` }, // Top-left
            { x: width, y: 0, anchor: "end", baseline: "hanging", coords: `(${displayBounds.x[1].toFixed(1)}, ${displayBounds.y[1].toFixed(1)})` }, // Top-right
            { x: 0, y: height, anchor: "start", baseline: "auto", coords: `(${displayBounds.x[0].toFixed(1)}, ${displayBounds.y[0].toFixed(1)})` }, // Bottom-left
            { x: width, y: height, anchor: "end", baseline: "auto", coords: `(${displayBounds.x[1].toFixed(1)}, ${displayBounds.y[0].toFixed(1)})` } // Bottom-right
        ];
        
        svg.selectAll(".corner-label")
            .data(cornerLabels)
            .enter()
            .append("text")
            .attr("class", "corner-label")
            .attr("x", d => d.x)
            .attr("y", d => d.y)
            .attr("dx", d => d.anchor === "start" ? 5 : -5) // Small offset from edge
            .attr("dy", d => d.baseline === "hanging" ? 15 : -5) // Small offset from edge
            .style("text-anchor", d => d.anchor)
            .style("dominant-baseline", d => d.baseline)
            .style("font-family", "monospace")
            .style("font-size", "12px")
            .style("fill", "black")
            .style("background", "rgba(255,255,255,0.8)")
            .text(d => d.coords);

        // Add padding to bounds for better visibility (independent for each dimension)
        {
            // final padding 12% for each dimension independently
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

        const colorScale = d3.scaleSequential(d3.interpolateRgbBasis(["blue", "green", "red"]))
            .domain([state.score_range.min, state.score_range.max]);

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
            .style('fill', d => d.min_score === null ? "#cccccc" : colorScale(d.min_score))
            .style('fill-opacity', 0.5)
            .style('stroke', 'black') // Add black border to regions
            .style('stroke-width', 1)
            .on("mouseover", (event, d) => {
                tooltip.transition().duration(200).style("opacity", .9);
                
                // Format min score in decimal format (same function as best score)
                const formatDecimal = (num) => {
                    if (num === 0) return '0';
                    return num.toFixed(20).replace(/\.?0+$/, '');
                };
                
                tooltip.html(`Region Bounds:<br/>  x: [${d.bounds.x[0].toFixed(2)}, ${d.bounds.x[1].toFixed(2)}]<br/>  y: [${d.bounds.y[0].toFixed(2)}, ${d.bounds.y[1].toFixed(2)}]<br/>Min Score: ${d.min_score ? formatDecimal(d.min_score) : 'N/A'}`)
                    .style("left", (event.pageX + 5) + "px")
                    .style("top", (event.pageY - 28) + "px");
            })
            .on("mouseout", () => {
                tooltip.transition().duration(500).style("opacity", 0);
            });

        regions.exit().remove();

        // Draw region information text as "carrying_capacity - actual_population"
        // Remove existing text elements first
        svg.selectAll('.region-text').remove();
        
        // Create text groups for each region to handle multi-colored text
        const regionTextGroups = svg.selectAll('.region-text-group')
            .data(state.regions, d => d.bounds.x[0] + "," + d.bounds.y[0]);

        const textGroups = regionTextGroups.enter()
            .append('g')
            .attr('class', 'region-text-group')
            .merge(regionTextGroups);

        textGroups.each(function(d) {
            // Count organisms in this region
            const organismsInRegion = (state.organisms || []).filter(organism =>
                organism.params.x >= d.bounds.x[0] && organism.params.x <= d.bounds.x[1] &&
                organism.params.y >= d.bounds.y[0] && organism.params.y <= d.bounds.y[1]
            ).length;
            
            const centerX = xScale(d.bounds.x[0]) + (xScale(d.bounds.x[1]) - xScale(d.bounds.x[0])) / 2;
            const centerY = yScale(d.bounds.y[1]) + (yScale(d.bounds.y[0]) - yScale(d.bounds.y[1])) / 2;
            
            // Clear existing elements in this group
            d3.select(this).selectAll('*').remove();
            
            // Create the full text to measure its width for background sizing
            const fullText = `${d.carrying_capacity} - ${organismsInRegion}`;
            
            // Add light gray background rectangle
            const padding = 4;
            const textWidth = fullText.length * 7; // Approximate text width
            const textHeight = 16;
            
            d3.select(this).append('rect')
                .attr('x', centerX - textWidth/2 - padding)
                .attr('y', centerY - textHeight/2 - padding)
                .attr('width', textWidth + 2*padding)
                .attr('height', textHeight + 2*padding)
                .attr('rx', 3) // Rounded corners
                .style('fill', 'rgba(240, 240, 240, 0.8)')
                .style('stroke', 'rgba(200, 200, 200, 0.5)')
                .style('stroke-width', 1);
            
            // Create three separate text elements for equal spacing
            const spacing = 8; // Space between elements
            
            // Add carrying capacity (always black)
            const capacityText = d3.select(this).append('text')
                .attr('x', centerX - spacing)
                .attr('y', centerY)
                .attr('dy', '0.35em')
                .style('text-anchor', 'end')
                .style('font-size', '12px')
                .style('fill', 'black')
                .style('font-weight', 'bold')
                .text(`${d.carrying_capacity}`);
            
            // Add hyphen (centered)
            d3.select(this).append('text')
                .attr('x', centerX)
                .attr('y', centerY)
                .attr('dy', '0.35em')
                .style('text-anchor', 'middle')
                .style('font-size', '12px')
                .style('fill', 'black')
                .style('font-weight', 'bold')
                .text('-');
            
            // Add actual population (green if <= capacity, red if > capacity)
            const populationColor = organismsInRegion <= d.carrying_capacity ? 'green' : 'red';
            d3.select(this).append('text')
                .attr('x', centerX + spacing)
                .attr('y', centerY)
                .attr('dy', '0.35em')
                .style('text-anchor', 'start')
                .style('font-size', '12px')
                .style('fill', populationColor)
                .style('font-weight', 'bold')
                .text(`${organismsInRegion}`);
        });

        regionTextGroups.exit().remove();

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
        const bestScore = world.training_run();
        
        // Update the display with current round and best score
        document.getElementById('round-counter').textContent = round;
        
        // Force decimal format even for very small numbers
        const formatDecimal = (num) => {
            if (num === 0) return '0';
            // Use toFixed with enough precision to capture the full number
            // JavaScript numbers have ~15-17 significant digits
            return num.toFixed(20).replace(/\.?0+$/, '');
        };
        
        document.getElementById('best-score').textContent = formatDecimal(bestScore);
        
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
