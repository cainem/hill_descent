// Smart number formatting utility functions
class NumberFormatter {
    /**
     * Formats a number with smart precision and scientific notation rules
     * @param {number} num - The number to format
     * @param {number} maxDecimals - Maximum decimal places for normal format (default: 6)
     * @returns {string} Formatted number string
     */
    static format(num, maxDecimals = 6) {
        if (num === null || num === undefined || !isFinite(num)) {
            return 'N/A';
        }

        if (num === 0) {
            return '0';
        }

        const absNum = Math.abs(num);
        const exponent = Math.floor(Math.log10(absNum));

        // Use normal format for exponents between -3 and +3 (inclusive)
        if (exponent >= -3 && exponent <= 3) {
            // For normal format, determine appropriate decimal places
            let decimals;
            if (absNum >= 100) {
                decimals = Math.max(0, maxDecimals - 3); // e.g., 123.456
            } else if (absNum >= 10) {
                decimals = Math.max(0, maxDecimals - 2); // e.g., 12.3456
            } else if (absNum >= 1) {
                decimals = Math.max(0, maxDecimals - 1); // e.g., 1.23456
            } else {
                decimals = maxDecimals + Math.abs(exponent); // e.g., 0.001234
            }
            return num.toFixed(decimals).replace(/\.?0+$/, '');
        } else {
            // Use scientific notation for numbers outside the range
            const precision = Math.max(2, maxDecimals - 1);
            return num.toExponential(precision);
        }
    }

    /**
     * Creates a formatted number element with tooltip showing full precision
     * @param {number} num - The number to format
     * @param {number} maxDecimals - Maximum decimal places for display (default: 6)
     * @returns {HTMLElement} Span element with formatted number and tooltip
     */
    static createFormattedElement(num, maxDecimals = 6) {
        const span = document.createElement('span');
        span.className = 'formatted-number';
        span.textContent = NumberFormatter.format(num, maxDecimals);

        if (num !== null && num !== undefined && isFinite(num)) {
            span.title = num.toExponential(15); // Full precision tooltip
            span.setAttribute('data-full-value', num.toString());
        }

        return span;
    }

    /**
     * Formats a number and returns both display text and tooltip text
     * @param {number} num - The number to format  
     * @param {number} maxDecimals - Maximum decimal places for display (default: 6)
     * @returns {object} Object with displayText and tooltipText properties
     */
    static formatWithTooltip(num, maxDecimals = 6) {
        return {
            displayText: NumberFormatter.format(num, maxDecimals),
            tooltipText: (num !== null && num !== undefined && isFinite(num)) ?
                num.toExponential(15) : 'N/A'
        };
    }
}

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
        this.selectedRegion = null; // Track selected region for click interactions

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
            // Region panel elements
            regionPanel: document.getElementById('region-panel'),
            closeRegionPanel: document.getElementById('close-region-panel'),
            regionXBounds: document.getElementById('region-x-bounds'),
            regionYBounds: document.getElementById('region-y-bounds'),
            regionMinScore: document.getElementById('region-min-score'),
            regionCapacity: document.getElementById('region-capacity'),
            regionPopulation: document.getElementById('region-population'),
            regionStatus: document.getElementById('region-status'),
            organismList: document.getElementById('organism-list'),
            // Organism modal elements
            organismModal: document.getElementById('organism-modal'),
            organismModalOverlay: document.getElementById('organism-modal-overlay'),
            closeOrganismModal: document.getElementById('close-organism-modal'),
            organismDetailId: document.getElementById('organism-detail-id'),
            organismDetailScore: document.getElementById('organism-detail-score'),
            organismDetailPosition: document.getElementById('organism-detail-position'),
            organismDetailAge: document.getElementById('organism-detail-age'),
            organismDetailStatus: document.getElementById('organism-detail-status'),
            organismDetailRegionKey: document.getElementById('organism-detail-region-key'),
            organismDetailParent1: document.getElementById('organism-detail-parent1'),
            organismDetailParent2: document.getElementById('organism-detail-parent2'),
            organismTreeview: document.getElementById('organism-treeview'),
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
            .attr('fill', '#f0f0f0')
            .style('cursor', 'default')
            .on('click', () => {
                // Clear region selection when clicking on background
                if (this.selectedRegion) {
                    this.hideRegionPanel();
                }
            });

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
        this.elements.closeRegionPanel.addEventListener('click', () => this.hideRegionPanel());
        this.elements.closeOrganismModal.addEventListener('click', () => this.hideOrganismModal());
        this.elements.organismModalOverlay.addEventListener('click', () => this.hideOrganismModal());
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
        this.elements.bestScoreSpan.innerHTML = '';
        this.elements.bestScoreSpan.appendChild(NumberFormatter.createFormattedElement(state.best_score));
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
            this.updateVisualization(webState);
        } catch (e) {
            console.error('Failed to parse world_state JSON', e);
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
        ctx.fillText(`Best: ${NumberFormatter.format(minScore)}`, padding, 20);
        ctx.fillText(`Worst: ${NumberFormatter.format(maxScore)}`, padding, 35);
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

    // Helper method to check if two regions are the same
    isRegionSelected(region) {
        return this.selectedRegion &&
            this.selectedRegion.bounds.x[0] === region.bounds.x[0] &&
            this.selectedRegion.bounds.x[1] === region.bounds.x[1] &&
            this.selectedRegion.bounds.y[0] === region.bounds.y[0] &&
            this.selectedRegion.bounds.y[1] === region.bounds.y[1];
    }

    // Show region detail panel
    showRegionPanel() {
        this.elements.regionPanel.style.display = 'block';
    }

    // Hide region detail panel
    hideRegionPanel() {
        this.elements.regionPanel.style.display = 'none';
        this.selectedRegion = null;
        if (this.currentState) {
            this.updateVisualization(this.currentState);
        }
    }

    // Update region panel with selected region data
    updateRegionPanel(region, organisms) {

        // Update region bounds with tooltips
        this.elements.regionXBounds.innerHTML = '';
        const xBoundsContainer = document.createElement('span');
        xBoundsContainer.textContent = '[';
        xBoundsContainer.appendChild(NumberFormatter.createFormattedElement(region.bounds.x[0]));
        xBoundsContainer.appendChild(document.createTextNode(', '));
        xBoundsContainer.appendChild(NumberFormatter.createFormattedElement(region.bounds.x[1]));
        xBoundsContainer.appendChild(document.createTextNode(']'));
        this.elements.regionXBounds.appendChild(xBoundsContainer);

        this.elements.regionYBounds.innerHTML = '';
        const yBoundsContainer = document.createElement('span');
        yBoundsContainer.textContent = '[';
        yBoundsContainer.appendChild(NumberFormatter.createFormattedElement(region.bounds.y[0]));
        yBoundsContainer.appendChild(document.createTextNode(', '));
        yBoundsContainer.appendChild(NumberFormatter.createFormattedElement(region.bounds.y[1]));
        yBoundsContainer.appendChild(document.createTextNode(']'));
        this.elements.regionYBounds.appendChild(yBoundsContainer);

        // Update region statistics with tooltip
        this.elements.regionMinScore.innerHTML = '';
        this.elements.regionMinScore.appendChild(NumberFormatter.createFormattedElement(region.min_score));
        this.elements.regionCapacity.textContent = region.carrying_capacity.toString();
        this.elements.regionPopulation.textContent = organisms.length.toString();

        // Update population status
        const isOverCapacity = organisms.length > region.carrying_capacity;
        const statusText = isOverCapacity ? 'Over Capacity' : 'Within Capacity';
        const statusColor = isOverCapacity ? '#d32f2f' : '#388e3c';
        this.elements.regionStatus.textContent = statusText;
        this.elements.regionStatus.style.color = statusColor;

        // Update organism list
        this.updateOrganismList(organisms);
    }

    // Update the organism list in the region panel
    updateOrganismList(organisms) {
        this.elements.organismList.innerHTML = '';

        if (organisms.length === 0) {
            const emptyMsg = document.createElement('div');
            emptyMsg.className = 'organism-item';
            emptyMsg.style.fontStyle = 'italic';
            emptyMsg.style.color = '#888';
            emptyMsg.textContent = 'No organisms in this region';
            this.elements.organismList.appendChild(emptyMsg);
            return;
        }

        // Sort organisms by score (lowest/best first)
        const sortedOrganisms = organisms.slice().sort((a, b) => {
            // Handle null/undefined scores
            if (a.score === null || a.score === undefined) return 1;
            if (b.score === null || b.score === undefined) return -1;
            return a.score - b.score; // Lower scores are better
        });

        sortedOrganisms.forEach(organism => {
            const organismEl = document.createElement('div');
            organismEl.className = 'organism-item';

            const header = document.createElement('div');
            header.className = 'organism-header';

            const idEl = document.createElement('span');
            idEl.className = 'organism-id';
            idEl.textContent = `ID: ${organism.id}`;

            const scoreEl = document.createElement('span');
            scoreEl.className = 'organism-score';
            if (organism.score !== null && organism.score !== undefined) {
                scoreEl.textContent = 'Score: ';
                scoreEl.appendChild(NumberFormatter.createFormattedElement(organism.score));
            } else {
                scoreEl.textContent = 'Score: N/A';
            }

            header.appendChild(idEl);
            header.appendChild(scoreEl);

            const details = document.createElement('div');
            details.className = 'organism-details';

            // Create X coordinate span
            const xSpan = document.createElement('span');
            xSpan.textContent = 'X: ';
            xSpan.appendChild(NumberFormatter.createFormattedElement(organism.params.x));

            // Create Y coordinate span  
            const ySpan = document.createElement('span');
            ySpan.textContent = 'Y: ';
            ySpan.appendChild(NumberFormatter.createFormattedElement(organism.params.y));

            // Create age span (no tooltip needed for integers)
            const ageSpan = document.createElement('span');
            ageSpan.textContent = `Age: ${organism.age}/${organism.max_age}`;

            // Create age percentage span
            const agePctSpan = document.createElement('span');
            agePctSpan.textContent = 'Age %: ';
            agePctSpan.appendChild(NumberFormatter.createFormattedElement((organism.age / organism.max_age) * 100));
            agePctSpan.appendChild(document.createTextNode('%'));

            details.appendChild(xSpan);
            details.appendChild(ySpan);
            details.appendChild(ageSpan);
            details.appendChild(agePctSpan);

            organismEl.appendChild(header);
            organismEl.appendChild(details);

            // Click handler for detailed organism view
            organismEl.addEventListener('click', () => {
                this.showOrganismModal();
                this.updateOrganismModal(organism);
            });

            this.elements.organismList.appendChild(organismEl);
        });
    }

    // Show organism detail modal
    showOrganismModal() {
        this.elements.organismModal.style.display = 'block';
        document.body.style.overflow = 'hidden'; // Prevent background scrolling
    }

    // Hide organism detail modal
    hideOrganismModal() {
        this.elements.organismModal.style.display = 'none';
        document.body.style.overflow = 'auto'; // Restore scrolling
    }

    // Update organism modal with detailed organism data
    updateOrganismModal(organism) {
        // Update organism summary
        this.elements.organismDetailId.textContent = organism.id.toString();

        // Score with tooltip
        this.elements.organismDetailScore.innerHTML = '';
        this.elements.organismDetailScore.appendChild(NumberFormatter.createFormattedElement(organism.score));

        // Position with tooltips
        this.elements.organismDetailPosition.innerHTML = '';
        const posContainer = document.createElement('span');
        posContainer.textContent = '(';
        posContainer.appendChild(NumberFormatter.createFormattedElement(organism.params.x));
        posContainer.appendChild(document.createTextNode(', '));
        posContainer.appendChild(NumberFormatter.createFormattedElement(organism.params.y));
        posContainer.appendChild(document.createTextNode(')'));
        this.elements.organismDetailPosition.appendChild(posContainer);

        this.elements.organismDetailAge.textContent = `${organism.age} / ${organism.max_age}`;
        this.elements.organismDetailStatus.textContent = organism.is_dead ? 'Dead' : 'Alive';
        this.elements.organismDetailRegionKey.textContent = organism.region_key ?
            `[${organism.region_key.join(', ')}]` : 'None';

        // Display parent information
        this.elements.organismDetailParent1.textContent = organism.parent_id_1 !== null && organism.parent_id_1 !== undefined ?
            organism.parent_id_1.toString() : 'None';
        this.elements.organismDetailParent2.textContent = organism.parent_id_2 !== null && organism.parent_id_2 !== undefined ?
            organism.parent_id_2.toString() : 'None';

        // Build and populate the treeview
        this.buildOrganismTreeview(organism);
    }

    // Build hierarchical treeview for organism genetic structure
    buildOrganismTreeview(organism) {
        this.elements.organismTreeview.innerHTML = '';

        // Determine organism type and parent info for display
        let organismType = 'Root';
        const hasParent1 = organism.parent_id_1 !== null && organism.parent_id_1 !== undefined;
        const hasParent2 = organism.parent_id_2 !== null && organism.parent_id_2 !== undefined;

        if (hasParent1 && hasParent2) {
            // Check if both parents are the same (self-fertilization)
            if (organism.parent_id_1 === organism.parent_id_2) {
                organismType = 'Sexual Offspring (Self-Fertilization)';
            } else {
                organismType = 'Sexual Offspring';
            }
        } else if (hasParent1) {
            // This should no longer occur with the new reproduction system
            organismType = 'Single Parent (Legacy)';
        }

        const treeData = {
            label: 'Organism',
            icon: '🦠',
            children: [
                {
                    label: 'Lineage',
                    icon: '🌳',
                    children: [
                        { label: 'Organism Type', icon: '🏷️', value: organismType, isLeaf: true },
                        {
                            label: 'Parent 1',
                            icon: hasParent1 ? '👨' : '❌',
                            value: hasParent1 ? organism.parent_id_1.toString() : 'None',
                            isLeaf: true
                        },
                        {
                            label: 'Parent 2',
                            icon: hasParent2 ? '👩' : '❌',
                            value: hasParent2 ? organism.parent_id_2.toString() : 'None',
                            isLeaf: true
                        }
                    ]
                },
                {
                    label: 'Phenotype',
                    icon: '🧬',
                    children: [
                        {
                            label: 'Expressed Values',
                            icon: '📊',
                            children: organism.phenotype.expressed_values.map((value, idx) => ({
                                label: `Value ${idx}`,
                                icon: '•',
                                value: NumberFormatter.format(value),
                                fullValue: value,
                                isLeaf: true
                            }))
                        },
                        {
                            label: 'Expressed Hash',
                            icon: '🔢',
                            value: organism.phenotype.expressed_hash.toString(),
                            isLeaf: true
                        },
                        {
                            label: 'System Parameters',
                            icon: '⚙️',
                            children: [
                                { label: 'M1 (Mutation Rate 1)', icon: '🎯', value: NumberFormatter.format(organism.phenotype.system_parameters.m1), fullValue: organism.phenotype.system_parameters.m1, isLeaf: true },
                                { label: 'M2 (Mutation Rate 2)', icon: '🎯', value: NumberFormatter.format(organism.phenotype.system_parameters.m2), fullValue: organism.phenotype.system_parameters.m2, isLeaf: true },
                                { label: 'M3 (Mutation Rate 3)', icon: '🎯', value: NumberFormatter.format(organism.phenotype.system_parameters.m3), fullValue: organism.phenotype.system_parameters.m3, isLeaf: true },
                                { label: 'M4 (Mutation Rate 4)', icon: '🎯', value: NumberFormatter.format(organism.phenotype.system_parameters.m4), fullValue: organism.phenotype.system_parameters.m4, isLeaf: true },
                                { label: 'M5 (Mutation Rate 5)', icon: '🎯', value: NumberFormatter.format(organism.phenotype.system_parameters.m5), fullValue: organism.phenotype.system_parameters.m5, isLeaf: true },
                                { label: 'Max Age', icon: '⏳', value: NumberFormatter.format(organism.phenotype.system_parameters.max_age), fullValue: organism.phenotype.system_parameters.max_age, isLeaf: true },
                                { label: 'Crossover Points', icon: '🔀', value: NumberFormatter.format(organism.phenotype.system_parameters.crossover_points), fullValue: organism.phenotype.system_parameters.crossover_points, isLeaf: true }
                            ]
                        },
                        {
                            label: 'Gamete 1',
                            icon: '🧬',
                            children: organism.phenotype.gamete1.loci.map((locus, idx) => ({
                                label: `Locus ${idx}`,
                                icon: '🔗',
                                children: [
                                    { label: 'Value', icon: '📊', value: NumberFormatter.format(locus.value), fullValue: locus.value, isLeaf: true },
                                    { label: 'Apply Flag', icon: '🏴', value: locus.apply_adjustment_flag.toString(), isLeaf: true },
                                    {
                                        label: 'Adjustment',
                                        icon: '⚡',
                                        children: [
                                            { label: 'Adjustment Value', icon: '📊', value: NumberFormatter.format(locus.adjustment.adjustment_value), fullValue: locus.adjustment.adjustment_value, isLeaf: true },
                                            { label: 'Direction', icon: '➡️', value: locus.adjustment.direction_of_travel, isLeaf: true },
                                            { label: 'Double/Half Flag', icon: '🔢', value: locus.adjustment.doubling_or_halving_flag.toString(), isLeaf: true },
                                            { label: 'Checksum', icon: '✔️', value: locus.adjustment.checksum.toString(), isLeaf: true }
                                        ]
                                    }
                                ]
                            }))
                        },
                        {
                            label: 'Gamete 2',
                            icon: '🧬',
                            children: organism.phenotype.gamete2.loci.map((locus, idx) => ({
                                label: `Locus ${idx}`,
                                icon: '🔗',
                                children: [
                                    { label: 'Value', icon: '📊', value: NumberFormatter.format(locus.value), fullValue: locus.value, isLeaf: true },
                                    { label: 'Apply Flag', icon: '🏴', value: locus.apply_adjustment_flag.toString(), isLeaf: true },
                                    {
                                        label: 'Adjustment',
                                        icon: '⚡',
                                        children: [
                                            { label: 'Adjustment Value', icon: '📊', value: NumberFormatter.format(locus.adjustment.adjustment_value), fullValue: locus.adjustment.adjustment_value, isLeaf: true },
                                            { label: 'Direction', icon: '➡️', value: locus.adjustment.direction_of_travel, isLeaf: true },
                                            { label: 'Double/Half Flag', icon: '🔢', value: locus.adjustment.doubling_or_halving_flag.toString(), isLeaf: true },
                                            { label: 'Checksum', icon: '✔️', value: locus.adjustment.checksum.toString(), isLeaf: true }
                                        ]
                                    }
                                ]
                            }))
                        }
                    ]
                }
            ]
        };

        this.renderTreeNode(treeData, this.elements.organismTreeview, 0);
    }

    // Render a tree node recursively
    renderTreeNode(node, container, level) {
        const nodeEl = document.createElement('div');
        nodeEl.className = `tree-node tree-level-${level % 5}`;

        const headerEl = document.createElement('div');
        headerEl.className = 'tree-node-header';

        // Add toggle button for nodes with children
        const toggleEl = document.createElement('div');
        toggleEl.className = 'tree-node-toggle';
        if (node.children && node.children.length > 0) {
            toggleEl.textContent = '+';
            toggleEl.style.cursor = 'pointer';
        } else {
            toggleEl.textContent = '';
        }

        // Add icon
        const iconEl = document.createElement('div');
        iconEl.className = 'tree-node-icon';
        iconEl.textContent = node.icon || '•';

        // Add label
        const labelEl = document.createElement('div');
        labelEl.className = 'tree-node-label';
        labelEl.textContent = node.label;

        // Add value if present
        const valueEl = document.createElement('div');
        valueEl.className = 'tree-node-value';
        if (node.value !== undefined) {
            if (node.fullValue !== undefined) {
                // Use formatted number with tooltip for numeric values
                const formattedElement = NumberFormatter.createFormattedElement(node.fullValue);
                valueEl.appendChild(formattedElement);
            } else {
                valueEl.textContent = node.value;
            }
        }

        headerEl.appendChild(toggleEl);
        headerEl.appendChild(iconEl);
        headerEl.appendChild(labelEl);
        if (node.value !== undefined) {
            headerEl.appendChild(valueEl);
        }

        nodeEl.appendChild(headerEl);

        // Add children container
        if (node.children && node.children.length > 0) {
            const childrenEl = document.createElement('div');
            childrenEl.className = 'tree-node-children';

            // Add click handler for toggle
            headerEl.addEventListener('click', () => {
                const isExpanded = childrenEl.classList.contains('expanded');
                if (isExpanded) {
                    childrenEl.classList.remove('expanded');
                    toggleEl.textContent = '+';
                } else {
                    childrenEl.classList.add('expanded');
                    toggleEl.textContent = '−';
                }
            });

            // Render children
            node.children.forEach(child => {
                if (child.isLeaf) {
                    const leafEl = document.createElement('div');
                    leafEl.className = `tree-node-leaf tree-level-${(level + 1) % 5}`;

                    const leafHeaderEl = document.createElement('div');
                    leafHeaderEl.className = 'tree-node-header';

                    const leafIconEl = document.createElement('div');
                    leafIconEl.className = 'tree-node-icon';
                    leafIconEl.textContent = child.icon || '•';

                    const leafLabelEl = document.createElement('div');
                    leafLabelEl.className = 'tree-node-label';
                    leafLabelEl.textContent = child.label;

                    const leafValueEl = document.createElement('div');
                    leafValueEl.className = 'tree-node-value';
                    if (child.fullValue !== undefined) {
                        // Use formatted number with tooltip for numeric values
                        const formattedElement = NumberFormatter.createFormattedElement(child.fullValue);
                        leafValueEl.appendChild(formattedElement);
                    } else {
                        leafValueEl.textContent = child.value || '';
                    }

                    leafHeaderEl.appendChild(leafIconEl);
                    leafHeaderEl.appendChild(leafLabelEl);
                    leafHeaderEl.appendChild(leafValueEl);
                    leafEl.appendChild(leafHeaderEl);

                    childrenEl.appendChild(leafEl);
                } else {
                    this.renderTreeNode(child, childrenEl, level + 1);
                }
            });

            nodeEl.appendChild(childrenEl);
        }

        container.appendChild(nodeEl);
    }

    // Ported rendering logic from main_old.js adjusted for server JSON
    updateVisualization(state) {
        if (!this.svg) return;

        // Store current state for region selection updates
        this.currentState = state;

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
            .attr('stroke', d => this.isRegionSelected(d) ? '#ff6600' : '#ccc')
            .attr('stroke-width', d => this.isRegionSelected(d) ? 3 : 0.5)
            .style('cursor', 'pointer')
            .on('click', (event, d) => {
                // Stop event propagation to prevent clearing selection
                event.stopPropagation();

                // Toggle selection
                if (this.isRegionSelected(d)) {
                    this.selectedRegion = null;
                    this.hideRegionPanel();
                } else {
                    this.selectedRegion = d;

                    // Find organisms in this region
                    const organismsInRegion = (state.organisms || []).filter(o => isOrgInRegion(o, d));

                    // Show and update the region panel
                    this.showRegionPanel();
                    this.updateRegionPanel(d, organismsInRegion);
                }

                // Update visualization to show selection state
                this.updateVisualization(this.currentState);
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
            { key: 'tl', x: 0, y: 0, label: `(${NumberFormatter.format(xMin)}, ${NumberFormatter.format(yMax)})`, anchor: 'start', vy: '1em' },
            { key: 'tr', x: width, y: 0, label: `(${NumberFormatter.format(xMax)}, ${NumberFormatter.format(yMax)})`, anchor: 'end', vy: '1em' },
            { key: 'bl', x: 0, y: height, label: `(${NumberFormatter.format(xMin)}, ${NumberFormatter.format(yMin)})`, anchor: 'start', vy: '-0.3em' },
            { key: 'br', x: width, y: height, label: `(${NumberFormatter.format(xMax)}, ${NumberFormatter.format(yMin)})`, anchor: 'end', vy: '-0.3em' },
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
            .on('click', (event, d) => {
                // Stop event propagation to prevent background clearing
                event.stopPropagation();

                // Show organism detail modal
                this.showOrganismModal();
                this.updateOrganismModal(d);
            })
            .style('cursor', 'pointer');

        orgSel.exit().remove();
    }


}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    new OptimizationUI();
});
