import init, { WasmWorld } from "../pkg/hill_descent.js";

async function main() {
    // 1. Initialize WASM
    await init();
    const world = WasmWorld.new();

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
            .style('fill-opacity', 0.5)
            .style('font-size', '12px')
            .style('fill', 'white')
            .text(d => d.carrying_capacity);

        regionTexts.exit().remove();

        // Render Organisms
        const organisms = svg.selectAll(".organism")
            .data(state.organisms);

        organisms.enter().append("circle")
            .attr("class", "organism")
            .attr("r", 3)
            .attr("fill", "black")
            .merge(organisms)
            .attr("cx", d => xScale(d.params.x))
            .attr("cy", d => yScale(d.params.y))
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
        world.training_run();
        updateVisualization();
    }

    // Initial render
    updateVisualization(); 

    // Run the simulation loop every second
    setInterval(simulationLoop, 1000);
}

main();
