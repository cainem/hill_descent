// API client for the hill descent server
class HillDescentClient {
    constructor(baseUrl = 'http://127.0.0.1:3000') {
        this.baseUrl = baseUrl;
        this.isRunning = false;
        this.autoInterval = null;
    }

    async startOptimization(populationSize = 100, eliteSize = 10) {
        const response = await fetch(`${this.baseUrl}/api/start`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                population_size: populationSize,
                elite_size: eliteSize,
                // You can add param_ranges here if needed
            }),
        });
        
        const data = await response.json();
        if (!data.success) {
            throw new Error(data.error || 'Failed to start optimization');
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
        this.autoInterval = null;
        this.bestScores = [];
        
        this.initializeElements();
        this.bindEvents();
    }

    initializeElements() {
        this.elements = {
            populationInput: document.getElementById('population'),
            eliteInput: document.getElementById('elite'),
            startBtn: document.getElementById('start-btn'),
            stepBtn: document.getElementById('step-btn'),
            autoBtn: document.getElementById('auto-btn'),
            resetBtn: document.getElementById('reset-btn'),
            epochSpan: document.getElementById('epoch'),
            bestScoreSpan: document.getElementById('best-score'),
            statusSpan: document.getElementById('status'),
            worldStatePre: document.getElementById('world-state'),
            chart: document.getElementById('optimization-chart'),
        };
    }

    bindEvents() {
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
            
            const state = await this.client.startOptimization(populationSize, eliteSize);
            this.updateUI(state);
            this.bestScores = [state.best_score];
            
            this.elements.startBtn.disabled = true;
            this.elements.stepBtn.disabled = false;
            this.elements.autoBtn.disabled = false;
            this.elements.resetBtn.disabled = false;
            this.isRunning = true;
            
            this.updateStatus('Optimization started');
            this.updateChart();
            
        } catch (error) {
            this.updateStatus(`Error: ${error.message}`);
            console.error('Start error:', error);
        }
    }

    async step() {
        try {
            const state = await this.client.stepOptimization();
            this.updateUI(state);
            this.bestScores.push(state.best_score);
            this.updateChart();
            
            if (state.at_resolution_limit) {
                this.updateStatus('Resolution limit reached!');
                this.stopAuto();
            }
            
        } catch (error) {
            this.updateStatus(`Error: ${error.message}`);
            console.error('Step error:', error);
        }
    }

    toggleAuto() {
        if (this.autoInterval) {
            this.stopAuto();
        } else {
            this.startAuto();
        }
    }

    startAuto() {
        this.autoInterval = setInterval(() => {
            this.step();
        }, 100); // Run every 100ms
        
        this.elements.autoBtn.textContent = 'Stop Auto';
        this.elements.stepBtn.disabled = true;
    }

    stopAuto() {
        if (this.autoInterval) {
            clearInterval(this.autoInterval);
            this.autoInterval = null;
        }
        
        this.elements.autoBtn.textContent = 'Auto Run';
        this.elements.stepBtn.disabled = false;
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
            this.elements.worldStatePre.textContent = 'No data available';
            this.updateStatus('Reset complete');
            this.clearChart();
            
        } catch (error) {
            this.updateStatus(`Error: ${error.message}`);
            console.error('Reset error:', error);
        }
    }

    updateUI(state) {
        this.elements.epochSpan.textContent = state.epoch;
        this.elements.bestScoreSpan.textContent = state.best_score.toFixed(6);
        this.elements.worldStatePre.textContent = state.world_state;
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
}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    new OptimizationUI();
});
