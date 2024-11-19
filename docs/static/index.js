// Function to fetch health services based on zip code and display results
async function fetchHealthServices(zip) {
    const resultsDiv = document.getElementById('results');
    const headerDiv = document.getElementById('resultsHeader');

    try {
        // Fetch data from the backend API
        const response = await fetch(`/services?zip=${zip}`);
        if (!response.ok) {
            throw new Error('Failed to fetch health services');
        }

        const data = await response.json(); // Parse the JSON response
        clearResults(); // Clear previous results before displaying new ones

        // Update the header with the number of results and the zip code
        const resultCount = data.length;
        headerDiv.innerHTML = `${resultCount} results found for ZIP code ${zip}`;

        // Display the health services
        if (data.length === 0) {
            resultsDiv.innerHTML = '<p>No health services found.</p>';
        } else {
            data.forEach(service => {
                // Create and append Bootstrap card for each service
                const card = createServiceCard(service);
                resultsDiv.appendChild(card);
            });
        }
    } catch (error) {
        console.error('Error fetching health services:', error);
    }
}

// Function to create a Bootstrap card for a service
function createServiceCard(service) {
    const card = document.createElement('div');
    card.classList.add('card', 'mb-3'); // Bootstrap card classes

    card.innerHTML = `
        <div class="row g-0">
            <div class="col-md-4">
                <img src="${service.imageUrl || 'default_image.png'}" class="img-fluid rounded-start" alt="${service.name}">
            </div>
            <div class="col-md-8">
                <div class="card-body">
                    <h5 class="card-title">${service.name}</h5>
                    <p class="card-text">${service.address}</p>
                </div>
            </div>
        </div>
    `;

    return card;
}

// Function to clear the search results
function clearResults() {
    const resultsDiv = document.getElementById('results');
    resultsDiv.innerHTML = ''; // Clear all content in the results div
    // Clear the text in the input field
    document.getElementById('zip').value = '';
}

// Event listener for the clear button
document.getElementById('clearBtn').addEventListener('click', function(e) {
    e.preventDefault(); // Prevent form from reloading the page
    clearResults(); // Call the clearResults function
});

// Event listener for the form submission
document.getElementById('zipForm').addEventListener('submit', function(e) {
    e.preventDefault(); // Prevent form from reloading the page

    const zip = document.getElementById('zip').value; // Get the zip code from the input
    fetchHealthServices(zip); // Call the fetchHealthServices function with the zip code
});