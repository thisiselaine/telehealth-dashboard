// Function to fetch health services based on zip code and display results
async function fetchHealthServices(zip) {
    const carouselInner = document.querySelector('#resultsCarousel .carousel-inner');
    const headerDiv = document.getElementById('resultsHeader');

    try {
        // Fetch data from the backend API
        const response = await fetch(`/services?zip=${zip}`);
        if (!response.ok) {
            throw new Error('Failed to fetch health services');
        }

        const data = await response.json();
        console.log('API Response:', data);

        const { coordinates, providers } = data;

        // Clear previous results
        clearResults();

        // Update the map
        updateMap(coordinates, providers);

        // Update the header
        const resultCount = providers ? providers.length : 0;
        headerDiv.innerHTML = `${resultCount} results found for ZIP code ${zip}`;

        // Handle empty results
        if (!providers || providers.length === 0) {
            carouselInner.innerHTML = '<div class="carousel-item"><p>No health services found.</p></div>';
            return;
        }

        // Populate carousel with cards
        providers.forEach((service) => {
            // Use createServiceCard to generate the card
            const card = createServiceCard(service);

            // Append the card to the carousel-inner
            carouselInner.appendChild(card);
        });
    } catch (error) {
        console.error('Error fetching health services:', error);
        headerDiv.innerHTML = '<p class="text-danger">Failed to load health services.</p>';
    }
}

function updateMap(coordinates, providers) {
    // Center the map on the provided coordinates
    if (!coordinates || !coordinates.lat || !coordinates.lng) {
        console.error('Invalid center coordinates:', coordinates);
        alert('Unable to center map due to invalid coordinates.');
        return;
    }

    // Unhide the map
    const mapDiv = document.getElementById('map');
    mapDiv.hidden = false;

    // Initialize the map
    const map = new google.maps.Map(mapDiv, {
        center: { lat: coordinates.lat, lng: coordinates.lng },
        zoom: 12,
    });

    // Initialize Google Maps Geocoder
    const geocoder = new google.maps.Geocoder();

    // Add markers for each provider using their addresses
    providers.forEach(provider => {
        if (!provider.address) {
            console.warn(`Skipping provider ${provider.name}: Missing address`);
            return;
        }

        // Geocode the provider's address to get coordinates
        geocoder.geocode({ address: provider.address }, (results, status) => {
            if (status === 'OK' && results[0]) {
                const location = results[0].geometry.location;

                // Create a marker using the geocoded location
                const marker = new google.maps.Marker({
                    position: location,
                    map: map,
                    title: provider.name,
                });

                const infoWindow = new google.maps.InfoWindow({
                    content: `
                        <div>
                            <h3>${provider.name}</h3>
                            <p>${provider.address}</p>
                            <p>${provider.phone ? `Phone: ${provider.phone}` : ''}</p>
                            <p>${provider.rating ? `Rating: ${provider.rating}` : ''}</p>
                        </div>
                    `,
                });

                marker.addListener('click', () => {
                    infoWindow.open(map, marker);
                });
            } else {
                console.warn(`Geocoding failed for provider ${provider.name}: ${status}`);
            }
        });
    });
}

// Function to create a Bootstrap card for a service
function createServiceCard(service) {
    const card = document.createElement('div');
    card.classList.add('card'); // Bootstrap card classes

    card.innerHTML = `
        <div class="row g-0">
            <div class="col-md-4">
                <img src="${service.imageUrl || 'default_image.png'}" class="img-fluid rounded-start" alt="${service.name}">
            </div>
            <div class="col-md-8">
                <div class="card-body">
                    <h5 class="card-title">${service.name}</h5>
                    <p class="card-text">${service.address}</p>
                    <p class="card-text">${service.phone ? `Phone: ${service.phone}` : ''}</p>
                    <p class="card-text">${service.rating ? `Rating: ${service.rating}` : ''}</p>
                </div>
            </div>
        </div>
    `;

    return card;
}

// Function to clear the search results
function clearResults() {
    const carouselInner = document.querySelector('#resultsCarousel .carousel-inner');
    const headerDiv = document.getElementById('resultsHeader');
    carouselInner.innerHTML = ''; // Clear carousel items
    headerDiv.innerHTML = ''; // Clear the header text
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

let map;

async function loadGoogleMaps() {
    try {
        const response = await fetch('/api-key');
        if (!response.ok) {
            throw new Error('Failed to fetch API key');
        }

        const apiKey = await response.text();
        const script = document.createElement('script');
        script.src = `https://maps.googleapis.com/maps/api/js?key=${apiKey}&callback=initMap`;
        script.async = true;
        script.defer = true;
        document.body.appendChild(script);
    } catch (error) {
        console.error('Error loading Google Maps API:', error);
    }
}


function initMap() {
    map = new google.maps.Map(document.getElementById('map'), {
        center: { lat: 37.7749, lng: -122.4194 },
        zoom: 12
    });
}

function populateCarousel(results) {
    const carouselInner = document.querySelector('#resultsCarousel .carousel-inner');
    carouselInner.innerHTML = ''; // Clear previous results

    results.forEach((result, index) => {
        const carouselItem = document.createElement('div');
        carouselItem.className = `carousel-item ${index === 0 ? 'active' : ''}`; // First item should be active
        carouselItem.innerHTML = `
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">${result.name}</h5>
                    <p class="card-text">${result.address}</p>
                    <p class="card-text">${result.phone ? `Phone: ${result.phone}` : ''}</p>
                    <p class="card-text">${result.rating ? `Rating: ${result.rating}` : ''}</p>
                </div>
            </div>
        `;
        carouselInner.appendChild(carouselItem);
    });
}

loadGoogleMaps();