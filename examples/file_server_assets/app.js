// JavaScript for Bevy WebGate Demo

async function loadData() {
    const button = document.querySelector('.cta-button');
    const dataDisplay = document.getElementById('data-display');
    
    button.disabled = true;
    button.textContent = 'Loading...';
    dataDisplay.className = 'api-response';
    dataDisplay.innerHTML = '<p>Fetching data from the API...</p>';
    
    try {
        let response = await fetch('/api/info');
        
        if (!response.ok) {
            response = await fetch('/static/data.json');
        }
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        
        dataDisplay.innerHTML = `
            <div>
                <h4 style="color: var(--success); margin-bottom: 1rem;">✅ API Response Loaded Successfully!</h4>
                <pre>${JSON.stringify(data, null, 2)}</pre>
            </div>
        `;
        dataDisplay.classList.add('loaded');
        
        button.textContent = 'Data Loaded Successfully!';
        setTimeout(() => {
            button.textContent = 'Reload Data';
            button.disabled = false;
        }, 2000);
        
    } catch (error) {
        dataDisplay.innerHTML = `
            <div>
                <h4 style="color: var(--error); margin-bottom: 1rem;">❌ Error Loading Data</h4>
                <p style="color: var(--text-secondary);">Error: ${error.message}</p>
                <p style="color: var(--text-muted); font-size: 0.875rem; margin-top: 0.5rem;">
                    Make sure the server is running and try again.
                </p>
            </div>
        `;
        dataDisplay.classList.add('error');
        button.textContent = 'Try Again';
        button.disabled = false;
    }
}

// Add some interactivity on page load
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Bevy WebGate File Server Demo loaded!');
    
    // Add hover effects to demo items
    const demoItems = document.querySelectorAll('.demo-item');
    demoItems.forEach(item => {
        item.addEventListener('mouseenter', function() {
            this.style.transform = 'translateY(-6px)';
            this.style.boxShadow = '0 15px 40px rgba(0, 0, 0, 0.4)';
        });
        
        item.addEventListener('mouseleave', function() {
            this.style.transform = 'translateY(-4px)';
            this.style.boxShadow = '0 10px 30px rgba(0, 0, 0, 0.3)';
        });
    });
    
    // Add click animation to feature list items
    const features = document.querySelectorAll('.feature-list li');
    features.forEach(feature => {
        feature.addEventListener('click', function() {
            this.style.transform = 'scale(1.02)';
            this.style.transition = 'transform 0.2s ease';
            setTimeout(() => {
                this.style.transform = 'scale(1)';
            }, 200);
        });
    });
    
    // Animate elements on scroll
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);
    
    // Observe all demo items and feature cards
    document.querySelectorAll('.demo-item, .feature-card, .code-example').forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(el);
    });
    
    // Add some visual feedback to the CTA button
    const ctaButton = document.querySelector('.cta-button');
    if (ctaButton) {
        ctaButton.addEventListener('mouseenter', function() {
            this.style.transform = 'translateY(-3px) scale(1.02)';
        });
        
        ctaButton.addEventListener('mouseleave', function() {
            if (!this.disabled) {
                this.style.transform = 'translateY(-2px) scale(1)';
            }
        });
    }
});
