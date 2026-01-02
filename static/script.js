// static/script.js
// This script generalizes the reference's script.js for n-pendulums. Adds n input handling: on change/load, generates dynamic form rows for m_i, L_i, θ_i.
// Collects params as comma-strings for backend. Animation: draws chain of rods (origin-pos1-pos2-...-posn), circles at each pos_k, red trace on end (last 100 frames).
// Matches reference: square canvas resize, grid, loop animation (speed=1), play/pause/reset (rewind only), auto-play on run.
// Assumptions: backend returns positions[t][2*k]=x_{k+1}, [2*k+1]=y; limit for scale; y-flip for canvas coords; trace on end effector.
// No JS plot (backend PNG); fetch error alerts. Runs animate() loop always, gated by isPlaying/animData.

let animData = null;  // Holds {positions: [...], n, limit} from backend.
let isPlaying = false;  // Play state.
let frameIdx = 0;  // Current frame.
let animationId = null;  // RAF id.
const speedMultiplier = 2;  // Frame skip, matches reference.

// DOM elements.
const canvas = document.getElementById('anim-canvas');  // Canvas ref.
const ctx = canvas.getContext('2d');  // 2D context.
const imgTraj = document.getElementById('traj-img');  // Image ref.
const btnRun = document.getElementById('btn-run');  // Run button.
const btnPlay = document.getElementById('btn-playpause');  // Play button.
const btnReset = document.getElementById('btn-reset');  // Reset.
const nInput = document.getElementById('n');  // n input.

// Resize canvas square on load/resize.
function resizeCanvas() {  // Matches reference.
    const parent = canvas.parentElement;  // Parent width.
    canvas.width = parent.clientWidth;  // Set width.
    canvas.height = parent.clientWidth;  // Square.
}
window.addEventListener('resize', resizeCanvas);  // Listen.
resizeCanvas();  // Initial.

// Generate dynamic fields on n change.
function generateFields() {  // Called on load/change.
    const n = parseInt(nInput.value);  // Get n.
    const container = document.getElementById('params-fields');  // Target div.
    container.innerHTML = '';  // Clear.
    for (let i = 1; i <= n; i++) {  // Loop 1 to n.
        // Defaults: match reference for n=2, 0° for >2.
        const mVal = 1.0;  // All m=1.
        const lVal = 1.0;  // All L=1.
        const thVal = (i === 1 ? 90 : i === 2 ? 45 : 0);  // θ defaults.
        container.innerHTML += ` _____________________________________
            <br><h4>Pendulum ${i} Parameters:</h4><br>
            <div class="form-row">
                <label>Mass ${i} (kg):</label>
                <input type="number" id="m${i}" value="${mVal}" step="0.1">
            </div>
            <div class="form-row">
                <label>Length ${i} (m):</label>
                <input type="number" id="L${i}" value="${lVal}" step="0.1">
            </div>
            <div class="form-row">
                <label>Initial θ${i} (deg):</label>
                <input type="number" id="th${i}" value="${thVal}" step="1">
            </div>
        `;
    }
}
nInput.addEventListener('change', generateFields);  // Listen for n change.
generateFields();  // Initial generate.

// Draw single frame: generalized to n rods/masses.
function drawFrame() {  // Matches reference structure.
    if (!animData) return;  // Guard.
    const w = canvas.width;  // Width.
    const h = canvas.height;  // Height.
    const limit = animData.limit;  // Scale base.
    const scale = (w / 2) / limit;  // Pixels per unit.
    const ox = w / 2;  // Origin x.
    const oy = h / 2;  // Origin y.
    ctx.clearRect(0, 0, w, h);  // Clear.
    // Grid (horizontal/vertical lines at origin).
    ctx.strokeStyle = '#eee';  // Light gray.
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, oy); ctx.lineTo(w, oy);  // Horz.
    ctx.moveTo(ox, 0); ctx.lineTo(ox, h);  // Vert.
    ctx.stroke();
    const i = frameIdx;  // Current idx.
    const positions = animData.positions[i];  // Current pos [x1,y1,x2,y2,...].
    // Trace: last 100 frames of end effector (n-1).
    const traceLen = 100;
    const startTrace = Math.max(0, i - traceLen);
    ctx.beginPath();
    ctx.strokeStyle = 'rgba(255, 0, 0, 0.5)';  // Red semi-trans.
    ctx.lineWidth = 1;
    for (let j = startTrace; j <= i; j++) {  // Loop trace frames.
        const posJ = animData.positions[j];
        const ex = posJ[2 * (animData.n - 1)] * scale + ox;  // End x.
        const ey = -posJ[2 * (animData.n - 1) + 1] * scale + oy;  // End y (flip).
        if (j === startTrace) ctx.moveTo(ex, ey);  // Start.
        else ctx.lineTo(ex, ey);  // Line.
    }
    ctx.stroke();  // Draw trace.
    // Rods: chain origin -> pos1 -> ... -> posn.
    ctx.beginPath();
    ctx.strokeStyle = 'black';  // Black.
    ctx.lineWidth = 2;
    let prevX = ox;  // Start at origin.
    let prevY = oy;
    ctx.moveTo(prevX, prevY);  // Move to origin.
    for (let k = 0; k < animData.n; k++) {  // For each segment.
        const x = positions[2 * k] * scale + ox;  // Pos x (0-based k=0 -> mass1).
        const y = -positions[2 * k + 1] * scale + oy;  // Flip y.
        ctx.lineTo(x, y);  // Line to pos.
        prevX = x;  // Update prev for next (but not used).
        prevY = y;
    }
    ctx.stroke();  // Draw chain.
    // Origin pivot.
    ctx.fillStyle = 'black';
    ctx.beginPath();
    ctx.arc(ox, oy, 3, 0, 2 * Math.PI);  // Small circle.
    ctx.fill();
    // Masses: circles at each pos_k.
    for (let k = 0; k < animData.n; k++) {  // Loop masses.
        const x = positions[2 * k] * scale + ox;
        const y = -positions[2 * k + 1] * scale + oy;
        ctx.beginPath();
        ctx.arc(x, y, 5, 0, 2 * Math.PI);  // Radius 5.
        ctx.fill();
    }
}

// Animation loop: RAF, advances if playing, loops.
function animate() {  // Recursive RAF.
    if (isPlaying && animData) {  // Guard.
        frameIdx += speedMultiplier;  // Advance.
        if (frameIdx >= animData.positions.length) {  // Loop.
            frameIdx = 0;
        }
        drawFrame();  // Draw.
    }
    animationId = requestAnimationFrame(animate);  // Next frame.
}

// Run button: collect params, fetch /simulate.
btnRun.addEventListener('click', () => {  // Click handler.
    const n = parseInt(nInput.value);  // n.
    let massesStr = '';  // Comma string.
    let lengthsStr = '';
    let anglesStr = '';
    for (let i = 1; i <= n; i++) {  // Collect.
        if (i > 1) {  // Comma after first.
            massesStr += ',';
            lengthsStr += ',';
            anglesStr += ',';
        }
        massesStr += document.getElementById(`m${i}`).value;
        lengthsStr += document.getElementById(`L${i}`).value;
        anglesStr += document.getElementById(`th${i}`).value;
    }
    const payload = {  // JSON body.
        n,
        masses: massesStr,
        lengths: lengthsStr,
        initial_angles: anglesStr,
        t_max: 60.0,  // Hardcode like reference.
        n_points: 8000
    };
    document.getElementById('loading-txt').style.display = 'block';  // Show loading.
    isPlaying = false;  // Stop current.
    btnPlay.innerText = "Play";
    fetch('/simulate', {  // POST.
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    })
    .then(async res => {
        const text = await res.text();
        if (!res.ok) throw new Error(text);
        return JSON.parse(text);
    }) // Parse JSON.
    .then(data => {  // Handle.
        document.getElementById('loading-txt').style.display = 'none';  // Hide.
        if (data.success) {  // Success.
            imgTraj.src = data.trajectory_image;  // Set image.
            animData = data.animation_data;  // Set data.
            frameIdx = 0;  // Reset.
            isPlaying = true;  // Auto-play.
            btnPlay.innerText = "Pause";
            drawFrame();  // Initial draw.
        } else {
            alert("Error: " + data.error);  // Alert, like reference.
        }
    })
    .catch(err => {  // Error.
        console.error(err);
        document.getElementById('loading-txt').style.display = 'none';
    });
});

// Play/pause: toggle; if no data and play, run first.
btnPlay.addEventListener('click', () => {
    if (!animData && !isPlaying) {  // No data: trigger run.
        btnRun.click();
        return;
    }
    isPlaying = !isPlaying;  // Toggle.
    btnPlay.innerText = isPlaying ? "Pause" : "Play";  // Update text.
});

// Reset: rewind to 0, keep playing.
btnReset.addEventListener('click', () => {
    frameIdx = 0;  // Reset idx.
    drawFrame();  // Redraw.
    // No pause, matches reference comment.
});

// Start empty loop.
animate();  // Initial call.