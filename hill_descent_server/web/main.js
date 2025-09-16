// API client for the hill descent server
class HillDescentClient {
    constructor(baseUrl = 'http://127.0.0.1:3000') {
        this.baseUrl = baseUrl;
        this.isRunning = false;
        this.autoInterval = null;
    }

    async startOptimization(populationSize = 100, eliteSize = 10, functionType = 'himmelblau') {
        const response = await fetch(`${this.baseUrl}/api/start`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                population_size: populationSize,
                elite_size: eliteSize,
                function_type: functionType,
            }),
        });

        const data = await response.json();
        if (!data.success) {
            throw new Error(data.error || 'Failed to start optimization');
        }

        return data.data;
    }

    async getFunctions() {
        const response = await fetch(`${this.baseUrl}/api/functions`);
        const data = await response.json();

        if (!data.success) {
            throw new Error(data.error || 'Failed to get functions');
        }

        return data.data;
    }

    async stepOptimization() {
        const response = await fetch(`${this.baseUrl}/api/step`, {
            method: 'POST',
        });

        const data = await response.json();
        if (!data.success) {
            throw new Error(data.error || 'Failed to step optimization');
        }

        return data.data;
    }

    async getState() {
        const response = await fetch(`${this.baseUrl}/api/state`);
        const data = await response.json();

        if (!data.success) {
            throw new Error(data.error || 'Failed to get state');
        }

        return data.data;
    }

    async reset() {
        const response = await fetch(`${this.baseUrl}/api/reset`, {
            method: 'POST',
        });

        const data = await response.json();
        if (!data.success) {
            throw new Error(data.error || 'Failed to reset');
        }

        return data.data;
    }
}

// UI Controller
class OptimizationUI {
    constructor() {
        this.client = new HillDescentClient();
        this.isRunning = false;
        this.autoInterval = null; // legacy, no longer used for auto-run
        this.isAutoRunning = false;
        this._stepInFlight = false;
        this.bestScores = [];
        this.availableFunctions = {};

        this.initializeElements();
        this.loadFunctions();
        this.initializeD3();
        this.bindEvents();
    }

    initializeElements() {
        this.elements = {
            functionSelect: document.getElementById('function-select'),
            populationInput: document.getElementById('population'),
            eliteInput: document.getElementById('elite'),
            startBtn: document.getElementById('start-btn'),
            stepBtn: document.getElementById('step-btn'),
            autoBtn: document.getElementById('auto-btn'),
            resetBtn: document.getElementById('reset-btn'),
            epochSpan: document.getElementById('epoch'),
            bestScoreSpan: document.getElementById('best-score'),
            statusSpan: document.getElementById('status'),
            currentFunctionSpan: document.getElementById('current-function'),
            functionInfo: document.getElementById('function-info'),
            functionDescription: document.getElementById('function-description'),
            functionDetails: document.getElementById('function-details'),
            visContainer: document.getElementById('visualization'),
            roundCounter: document.getElementById('round-counter'),
            tooltip: document.getElementById('tooltip'),
        };
    }

    async loadFunctions() {
        try {
            this.availableFunctions = await this.client.getFunctions();
            this.populateFunctionSelect();
        } catch (error) {
            console.error('Failed to load functions:', error);
            this.elements.functionSelect.innerHTML = '<option value="">Error loading functions</option>';
        }
    }

    populateFunctionSelect() {
        const select = this.elements.functionSelect;
        select.innerHTML = '';

        // Add default option
        const defaultOption = document.createElement('option');
        defaultOption.value = '';
        defaultOption.textContent = 'Select a function...';
        select.appendChild(defaultOption);

        // Add function options
        Object.entries(this.availableFunctions).forEach(([key, info]) => {
            const option = document.createElement('option');
            option.value = key.toLowerCase();
            option.textContent = info.name;
            select.appendChild(option);
        });

        // Set default selection to Himmelblau if available
        if (this.availableFunctions.himmelblau || this.availableFunctions.Himmelblau) {
            select.value = 'himmelblau';
            this.onFunctionChange();
        }
    }

    onFunctionChange() {
        const selectedFunction = this.elements.functionSelect.value;
        if (!selectedFunction) {
            this.elements.functionInfo.style.display = 'none';
            return;
        }

        // Find the function info (case-insensitive)
        const functionInfo = Object.entries(this.availableFunctions).find(([key, _]) =>
            key.toLowerCase() === selectedFunction.toLowerCase()
        )?.[1];

        if (functionInfo) {
            this.elements.functionDescription.textContent = functionInfo.description;

            let details = `<strong>Parameter ranges:</strong><br>`;
            functionInfo.param_ranges.forEach((range, index) => {
                details += `Dimension ${index + 1}: [${range[0]}, ${range[1]}]<br>`;
            });

            if (functionInfo.global_minimum) {
                details += `<br><strong>Global minimum:</strong> (${functionInfo.global_minimum[0]}, ${functionInfo.global_minimum[1]})`;
            }

            this.elements.functionDetails.innerHTML = details;
            this.elements.functionInfo.style.display = 'block';
        }
    }

    initializeD3() {
        // Setup D3 SVG and basic elements similar to old UI
        const margin = { top: 20, right: 20, bottom: 30, left: 40 };
        const width = 800 - margin.left - margin.right;
        const height = 800 - margin.top - margin.bottom;

        this.d3cfg = { margin, width, height };

        this.svg = d3.select('#visualization').append('svg')
            .attr('width', width + margin.left + margin.right)
            .attr('height', height + margin.top + margin.bottom)
            .append('g')
            .attr('transform', `translate(${margin.left},${margin.top})`);

        this.svg.append('rect')
            .attr('class', 'world-background')
            .attr('x', 0)
            .attr('y', 0)
            .attr('width', width)
            .attr('height', height)
            .attr('fill', '#f0f0f0');

        // Explicit layer groups to control z-order
        // Order: regions (bottom) -> organisms (middle) -> overlays (top)
        this.gRegions = this.svg.append('g').attr('class', 'layer-regions');
        this.gOrganisms = this.svg.append('g').attr('class', 'layer-organisms');
        this.gOverlay = this.svg.append('g').attr('class', 'layer-overlay').style('pointer-events', 'none');
        this.gCorners = this.svg.append('g').attr('class', 'layer-corners').style('pointer-events', 'none');

        this.xScale = d3.scaleLinear().range([0, width]);
        this.yScale = d3.scaleLinear().range([height, 0]);
    }

    bindEvents() {
        this.elements.functionSelect.addEventListener('change', () => this.onFunctionChange());
        this.elements.startBtn.addEventListener('click', () => this.start());
        this.elements.stepBtn.addEventListener('click', () => this.step());
        this.elements.autoBtn.addEventListener('click', () => this.toggleAuto());
        this.elements.resetBtn.addEventListener('click', () => this.reset());
    }

    async start() {
        try {
            this.updateStatus('Starting optimization...');

            const populationSize = parseInt(this.elements.populationInput.value);
            const eliteSize = parseInt(this.elements.eliteInput.value);
            const functionType = this.elements.functionSelect.value;

            if (!functionType) {
                alert('Please select an optimization function first.');
                return;
            }

            const state = await this.client.startOptimization(populationSize, eliteSize, functionType);
            this.updateUI(state);
            this.bestScores = [state.best_score];

            this.elements.startBtn.disabled = true;
            this.elements.stepBtn.disabled = false;
            this.elements.autoBtn.disabled = false;
            this.elements.resetBtn.disabled = false;
            this.isRunning = true;

            this.updateStatus('Optimization started');

        } catch (error) {
            this.updateStatus(`Error: ${error.message}`);
            console.error('Start error:', error);
        }
    }

    async step() {
        if (this._stepInFlight) {
            return; // prevent overlapping requests
        }
        this._stepInFlight = true;
        try {
            const data = await this.client.stepOptimization();
            this.updateUI(data);
            this.bestScores.push(data.best_score);

            if (data.at_resolution_limit) {
                this.updateStatus('Resolution limit reached!');
                this.stopAuto();
            }

        } catch (err) {
            console.error(err);
            this.updateStatus('Step failed');
        } finally {
            this._stepInFlight = false;
        }
    }

    toggleAuto() {
        if (this.isAutoRunning) {
            this.stopAuto();
        } else {
            this.startAuto();
        }
    }

    startAuto() {
        if (this.isAutoRunning) return;
        this.isAutoRunning = true;
        this.elements.autoBtn.textContent = 'Stop Auto';
        this.elements.stepBtn.disabled = true;
        this.runAutoLoop();
    }

    stopAuto() {
        this.isAutoRunning = false;
        this.elements.autoBtn.textContent = 'Auto Run';
        this.elements.stepBtn.disabled = false;
    }

    async runAutoLoop() {
        const sleep = (ms) => new Promise(res => setTimeout(res, ms));
        while (this.isAutoRunning) {
            await this.step(); // waits for response before continuing
            await sleep(500); // pacing
        }
    }

    async reset() {
        try {
            this.stopAuto();
            await this.client.reset();

            this.elements.startBtn.disabled = false;
            this.elements.stepBtn.disabled = true;
            this.elements.autoBtn.disabled = true;
            this.elements.resetBtn.disabled = true;
            this.isRunning = false;
            this.bestScores = [];

            this.elements.epochSpan.textContent = '-';
            this.elements.bestScoreSpan.textContent = '-';
            this.updateStatus('Reset complete');
            this.clearVisualization();

        } catch (error) {
            this.updateStatus(`Error: ${error.message}`);
            console.error('Reset error:', error);
        }
    }

    updateUI(state) {
        this.elements.epochSpan.textContent = state.epoch;
        this.elements.bestScoreSpan.textContent = state.best_score.toFixed(6);
        this.elements.roundCounter.textContent = state.epoch;

        // Update current function display
        if (state.function_type) {
            const functionInfo = Object.entries(this.availableFunctions).find(([key, _]) =>
                key.toLowerCase() === state.function_type.toLowerCase()
            )?.[1];
            this.elements.currentFunctionSpan.textContent = functionInfo ? functionInfo.name : state.function_type;
        }

        // Render D3 visualization from web-shaped JSON
        try {
            const webState = JSON.parse(state.world_state);
            // Log state with parsed world_state as JSON for better browser console viewing
            console.log('[UI] StateResponse from server', {
                ...state,
                world_state: webState
            });
            console.log('[UI] Parsed webState', {
                world_bounds: webState.world_bounds,
                score_range: webState.score_range,
                regions: webState.regions ? webState.regions.length : 0,
                organisms: webState.organisms ? webState.organisms.length : 0,
            });
            this.updateVisualization(webState);
        } catch (e) {
            console.error('Failed to parse world_state JSON', e);
            // Fallback to original logging if parsing fails
            console.log('[UI] StateResponse from server (raw)', state);
        }
    }

    updateStatus(message) {
        this.elements.statusSpan.textContent = message;
    }

    updateChart() {
        // Simple canvas-based chart for best scores over time
        const canvas = this.elements.chart;
        const ctx = canvas.getContext('2d');

        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        if (this.bestScores.length < 2) return;

        // Set up chart dimensions
        const padding = 50;
        const chartWidth = canvas.width - 2 * padding;
        const chartHeight = canvas.height - 2 * padding;

        // Find min/max values
        const minScore = Math.min(...this.bestScores);
        const maxScore = Math.max(...this.bestScores);
        const scoreRange = maxScore - minScore || 1;

        // Draw axes
        ctx.strokeStyle = '#ccc';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(padding, padding);
        ctx.lineTo(padding, canvas.height - padding);
        ctx.lineTo(canvas.width - padding, canvas.height - padding);
        ctx.stroke();

        // Draw grid lines
        ctx.strokeStyle = '#eee';
        for (let i = 1; i < 10; i++) {
            const y = padding + (chartHeight * i / 10);
            ctx.beginPath();
            ctx.moveTo(padding, y);
            ctx.lineTo(canvas.width - padding, y);
            ctx.stroke();
        }

        // Draw the line
        ctx.strokeStyle = '#007acc';
        ctx.lineWidth = 2;
        ctx.beginPath();

        for (let i = 0; i < this.bestScores.length; i++) {
            const x = padding + (chartWidth * i / (this.bestScores.length - 1));
            const y = canvas.height - padding - (chartHeight * (this.bestScores[i] - minScore) / scoreRange);

            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        }
        ctx.stroke();

        // Draw labels
        ctx.fillStyle = '#333';
        ctx.font = '12px Arial';
        ctx.fillText(`Best: ${minScore.toFixed(6)}`, padding, 20);
        ctx.fillText(`Worst: ${maxScore.toFixed(6)}`, padding, 35);
        ctx.fillText(`Epochs: ${this.bestScores.length}`, canvas.width - 100, 20);
    }

    clearChart() {
        const canvas = this.elements.chart;
        const ctx = canvas.getContext('2d');
        ctx.clearRect(0, 0, canvas.width, canvas.height);
    }

    clearVisualization() {
        if (this.svg) {
            this.svg.selectAll('.region').remove();
            this.svg.selectAll('.organism').remove();
            this.svg.selectAll('.region-label').remove();
        }
    }

    // Ported rendering logic from main_old.js adjusted for server JSON
    updateVisualization(state) {
        if (!this.svg) return;

        const { width, height } = this.d3cfg;

        // Determine display bounds prioritizing active regions
        let displayBounds = {
            x: [...state.world_bounds.x],
            y: [...state.world_bounds.y],
        };

        const isOrgInRegion = (org, r) => {
            const [x0, x1] = r.bounds.x;
            const [y0, y1] = r.bounds.y;
            return (
                org.params.x >= x0 && org.params.x <= x1 &&
                org.params.y >= y0 && org.params.y <= y1
            );
        };

        const activeRegions = (state.regions || []).filter(r => {
            if (r.carrying_capacity > 0) return true;
            return (state.organisms || []).some(o => isOrgInRegion(o, r));
        });

        if (activeRegions.length > 0) {
            const allX = activeRegions.flatMap(r => r.bounds.x);
            const allY = activeRegions.flatMap(r => r.bounds.y);
            displayBounds.x[0] = d3.min(allX);
            displayBounds.x[1] = d3.max(allX);
            displayBounds.y[0] = d3.min(allY);
            displayBounds.y[1] = d3.max(allY);
        } else if (state.organisms && state.organisms.length > 0) {
            displayBounds.x[0] = d3.min(state.organisms, o => o.params.x);
            displayBounds.x[1] = d3.max(state.organisms, o => o.params.x);
            displayBounds.y[0] = d3.min(state.organisms, o => o.params.y);
            displayBounds.y[1] = d3.max(state.organisms, o => o.params.y);
        }

        // Pad function (12%)
        const pad = (min, max) => {
            const range = max - min;
            const padding = 0.12 * range;
            return [min - padding, max + padding];
        };
        const [xMin, xMax] = pad(displayBounds.x[0], displayBounds.x[1]);
        const [yMin, yMax] = pad(displayBounds.y[0], displayBounds.y[1]);

        this.xScale.domain([xMin, xMax]);
        this.yScale.domain([yMin, yMax]);

        // Color scale for region min_score
        const scoreMin = state.score_range.min;
        const scoreMax = state.score_range.max;

        // Use logarithmic scale for better color distribution across large score ranges
        const logScoreMin = Math.log10(scoreMin);
        const logScoreMax = Math.log10(scoreMax);
        const colorScale = d3.scaleSequential(d3.interpolateRdYlGn)
            .domain([logScoreMax, logScoreMin]); // lower scores are greener



        // Render regions
        const regionsSel = this.gRegions.selectAll('.region')
            .data(state.regions || [], d => `${d.bounds.x}-${d.bounds.y}`);

        const fmtDec = (num) => {
            if (num === 0) return '0';
            return Number(num).toFixed(20).replace(/\.?0+$/, '');
        };

        regionsSel.enter().append('rect')
            .attr('class', 'region')
            .merge(regionsSel)
            .attr('x', d => this.xScale(d.bounds.x[0]))
            .attr('y', d => this.yScale(d.bounds.y[1]))
            .attr('width', d => this.xScale(d.bounds.x[1]) - this.xScale(d.bounds.x[0]))
            .attr('height', d => this.yScale(d.bounds.y[0]) - this.yScale(d.bounds.y[1]))
            .attr('fill', d => {
                // If no min_score yet, use the worst color from the scale (highest score)
                if (d.min_score == null) return colorScale(logScoreMax);
                return colorScale(Math.log10(d.min_score));
            })
            .attr('stroke', '#ccc')
            .attr('stroke-width', 0.5)
            .on('mouseover', (event, d) => {
                const orgCount = (state.organisms || []).filter(o => isOrgInRegion(o, d)).length;
                console.log('[UI] Region hover', { region: d, carrying_capacity: d.carrying_capacity, min_score: d.min_score, orgCount });
                const tooltip = d3.select('#tooltip');
                tooltip.transition().duration(200).style('opacity', 0.9);
                tooltip.html(
                    `Region Bounds:<br/>  x: [${d.bounds.x[0].toFixed(2)}, ${d.bounds.x[1].toFixed(2)}]` +
                    `<br/>  y: [${d.bounds.y[0].toFixed(2)}, ${d.bounds.y[1].toFixed(2)}]` +
                    `<br/>Min Score: ${d.min_score != null ? fmtDec(d.min_score) : 'N/A'}` +
                    `<br/>Carrying Capacity: ${d.carrying_capacity}` +
                    `<br/>Organisms: ${orgCount}`
                )
                    .style('left', (event.pageX + 5) + 'px')
                    .style('top', (event.pageY - 28) + 'px');
            })
            .on('mouseout', () => {
                d3.select('#tooltip').transition().duration(500).style('opacity', 0);
            });

        regionsSel.exit().remove();

        // Capacity-pop overlays
        const overlay = this.gOverlay.selectAll('.region-label')
            .data(state.regions || [], d => `${d.bounds.x}-${d.bounds.y}`);

        overlay.enter().append('g')
            .attr('class', 'region-label')
            .merge(overlay)
            .each((d, i, nodes) => {
                const sel = d3.select(nodes[i]);
                sel.selectAll('*').remove();
                const cx = this.xScale((d.bounds.x[0] + d.bounds.x[1]) / 2);
                const cy = this.yScale((d.bounds.y[0] + d.bounds.y[1]) / 2);

                const orgInRegion = (state.organisms || []).filter(o => isOrgInRegion(o, d)).length;
                const boxWidth = 40, boxHeight = 18;

                sel.append('rect')
                    .attr('x', cx - boxWidth / 2)
                    .attr('y', cy - boxHeight / 2)
                    .attr('width', boxWidth)
                    .attr('height', boxHeight)
                    .attr('rx', 3)
                    .style('fill', 'rgba(240,240,240,0.8)')
                    .style('stroke', 'rgba(200,200,200,0.5)')
                    .style('stroke-width', 1);

                const spacing = 8;
                sel.append('text')
                    .attr('x', cx - spacing)
                    .attr('y', cy)
                    .attr('dy', '0.35em')
                    .style('text-anchor', 'end')
                    .style('font-size', '12px')
                    .style('fill', 'black')
                    .style('font-weight', 'bold')
                    .text(`${d.carrying_capacity}`);

                sel.append('text')
                    .attr('x', cx)
                    .attr('y', cy)
                    .attr('dy', '0.35em')
                    .style('text-anchor', 'middle')
                    .style('font-size', '12px')
                    .style('fill', 'black')
                    .style('font-weight', 'bold')
                    .text('-');

                const popColor = orgInRegion <= d.carrying_capacity ? 'green' : 'red';
                sel.append('text')
                    .attr('x', cx + spacing)
                    .attr('y', cy)
                    .attr('dy', '0.35em')
                    .style('text-anchor', 'start')
                    .style('font-size', '12px')
                    .style('fill', popColor)
                    .style('font-weight', 'bold')
                    .text(`${orgInRegion}`);
            });

        overlay.exit().remove();

        // Corner coordinate labels (show current view bounds)
        const corners = [
            { key: 'tl', x: 0, y: 0, label: `(${xMin.toFixed(2)}, ${yMax.toFixed(2)})`, anchor: 'start', vy: '1em' },
            { key: 'tr', x: width, y: 0, label: `(${xMax.toFixed(2)}, ${yMax.toFixed(2)})`, anchor: 'end', vy: '1em' },
            { key: 'bl', x: 0, y: height, label: `(${xMin.toFixed(2)}, ${yMin.toFixed(2)})`, anchor: 'start', vy: '-0.3em' },
            { key: 'br', x: width, y: height, label: `(${xMax.toFixed(2)}, ${yMin.toFixed(2)})`, anchor: 'end', vy: '-0.3em' },
        ];

        const cornerSel = this.gCorners.selectAll('.corner-label').data(corners, d => d.key);
        cornerSel.enter()
            .append('text')
            .attr('class', 'corner-label')
            .merge(cornerSel)
            .attr('x', d => d.x + (d.anchor === 'start' ? 4 : -4))
            .attr('y', d => d.y)
            .attr('dy', d => d.vy)
            .style('font-size', '11px')
            .style('fill', '#333')
            .style('text-anchor', d => d.anchor)
            .text(d => d.label);
        cornerSel.exit().remove();

        // Organisms
        const ageColor = d3.scaleSequential(d3.interpolateRgbBasis(['blue', 'green', 'red']))
            .domain([0, 1]);

        const orgSel = this.gOrganisms.selectAll('.organism').data(state.organisms || []);

        orgSel.enter().append('circle')
            .attr('class', 'organism')
            .attr('r', 4)
            .merge(orgSel)
            .attr('fill', d => ageColor(d.age / d.max_age))
            .attr('stroke', '#ffffff')
            .attr('stroke-width', 0.75)
            .attr('cx', d => this.xScale(d.params.x))
            .attr('cy', d => this.yScale(d.params.y))
            .on('mouseover', (event, d) => {
                console.log('[UI] Organism hover', d);
                const tooltip = d3.select('#tooltip');
                tooltip.transition().duration(200).style('opacity', 0.9);
                tooltip.html(
                    `Organism ID: ${d.id}<br/>  x: ${d.params.x.toFixed(2)}<br/>  y: ${d.params.y.toFixed(2)}<br/>Age: ${d.age}<br/>Max Age: ${d.max_age} (raw: ${d.raw_max_age.toFixed(3)})<br/>Score: ${d.score !== null && d.score !== undefined ? d.score.toFixed(6) : 'N/A'}`
                )
                    .style('left', (event.pageX + 5) + 'px')
                    .style('top', (event.pageY - 28) + 'px');
            })
            .on('mouseout', () => {
                d3.select('#tooltip').transition().duration(500).style('opacity', 0);
            });

        orgSel.exit().remove();
    }


}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    new OptimizationUI();
});
