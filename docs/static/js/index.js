// Function to fetch health services based on zip code and display results
async function fetchHealthServices(location, serviceType, useCurLocation) {
    const headerDiv = document.getElementById('resultsHeader');

    if (!serviceType) {
        const serviceTypeDropdown = document.getElementById('serviceType');
        serviceTypeDropdown.classList.add('is-invalid');
        headerDiv.innerHTML = '<p class="text-danger">Please specify a service type.</p>';
        return;
    } else {
        document.getElementById('serviceType').classList.remove('is-invalid');
    }

    try {
        let response = null;
        if (!useCurLocation) {
            response = await fetch(`/services?zip=${location}&service_type=${serviceType}`);
        } else {
            response = await fetch(`/services?lat=${location.lat}&lng=${location.lng}&service_type=${serviceType}`);
        }
        if (!response.ok) {
            throw new Error('Failed to fetch health services');
        }

        const data = await response.json();
        const { coordinates, providers } = data;

        // Update the map and get markers
        clearResults();
        const markers = await updateMap(coordinates, providers, useCurLocation);

        const resultCount = providers ? providers.length : 0;

        if (useCurLocation) {
            headerDiv.innerHTML = `${resultCount} results found at current location`;
        } else {
            headerDiv.innerHTML = `${resultCount} results found for ZIP code ${location}`;
        }
        if (!providers || providers.length === 0) {
            carouselInner.innerHTML = '<div class="carousel-item"><p>No health services found.</p></div>';
            return;
        }

        populateCarousel(providers, markers);
    } catch (error) {
        console.error('Error fetching health services:', error);
        headerDiv.innerHTML = '<p class="text-danger">Failed to load health services.</p>';
    }
}

function populateCarousel(providers, markers) {
    const carouselInner = document.querySelector('#resultsCarousel .carousel-inner');
    const carouselDiv = document.getElementById('resultsCarousel');
    carouselDiv.hidden = false;

    // Sort providers by rating (descending order)
    providers.sort((a, b) => {
        // Handle cases where ratings are missing
        const ratingA = a.rating || 0;
        const ratingB = b.rating || 0;
        return ratingB - ratingA;
    });

    // Populate carousel and link cards to markers
    providers.forEach((service) => {
        const card = createServiceCard(service);

        // Find the corresponding marker
        const markerEntry = markers.find(({ provider }) => provider.name === service.name && provider.address === service.address);

        if (markerEntry) {
            card.addEventListener('click', () => {
                // Close all open InfoWindows
                markers.forEach(({ infoWindow }) => infoWindow.close());

                // Open the corresponding InfoWindow
                markerEntry.infoWindow.open(markerEntry.marker.getMap(), markerEntry.marker);
                currentInfoWindow = markerEntry.infoWindow;

                // Pan to the marker's position
                markerEntry.marker.getMap().panTo(markerEntry.marker.getPosition());
            });
        }

        carouselInner.appendChild(card);
    });
}



function updateMap(coordinates, providers, useCurLocation) {
    if (!coordinates || !coordinates.lat || !coordinates.lng) {
        console.error('Invalid center coordinates:', coordinates);
        alert('Unable to center map due to invalid coordinates.');
        return;
    }

    const mapDiv = document.getElementById('map');
    mapDiv.hidden = false;

    const map = new google.maps.Map(mapDiv, {
        center: { lat: coordinates.lat, lng: coordinates.lng },
        zoom: 12,
    });

    if (useCurLocation) {
        showUserLocation(map, coordinates);
    }

    const geocoder = new google.maps.Geocoder();

    const markers = []; // Proper array to store markers

    // Wrap geocoding in promises to handle async operations
    const geocodePromises = providers.map((provider) => {
                if (!provider.address) {
                    console.warn(`Skipping provider ${provider.name}: Missing address`);
                    return Promise.resolve(); // Resolve immediately for missing addresses
                }

                return new Promise((resolve) => {
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
                        if (currentInfoWindow) {
                            currentInfoWindow.close();
                        }
                        infoWindow.open(map, marker);
                        currentInfoWindow = infoWindow;
                    });

                    // Add to markers array
                    markers.push({ marker, infoWindow, provider });
                } else {
                    console.warn(`Geocoding failed for provider ${provider.name}: ${status}`);
                }

                resolve(); // Resolve the promise
            });
        });
    });

    // Wait for all geocoding tasks to complete
    return Promise.all(geocodePromises).then(() => {
        console.log('All geocoding completed. Markers:', markers);
        return markers; // Return the fully populated markers array
    });
}

// Function to create a Bootstrap card for a service
function createServiceCard(service) {
    const card = document.createElement('div');
    card.classList.add('card'); // Bootstrap card classes

    card.innerHTML = `
        <div class="row g-0">
            <div class="col-md-4">
                <img src="${service.photo_url || 'default_image.png'}" class="img-fluid rounded-start" alt="${service.name}">
            </div>
            <div class="col-md-8">
                <div class="card-body">
                    <h5 class="card-title">${service.name}</h5>
                    <p class="card-text">${service.address}</p>
                    <p class="card-text">${service.phone ? `Phone: ${service.phone}` : ''}</p>
                    <p class = ""card-text>${service.rating ? `Rating: ${service.rating.toFixed(1)}` : ''}</p>
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
    const carouselDiv = document.getElementById('resultsCarousel');
    const mapDiv = document.getElementById('map');
    mapDiv.hidden = true;
    carouselDiv.hidden=true;
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
    const serviceType = document.getElementById('serviceType').value;
    fetchHealthServices(zip, serviceType, false); // Call the fetchHealthServices function with the zip code
});

// Event listener for the form submission
document.getElementById('searchBtn').addEventListener('click', function(e) {
    e.preventDefault(); // Prevent form from reloading the page
    const serviceType = document.getElementById('serviceType').value;
    const zip = document.getElementById('zip').value; // Get the zip code from the input
    fetchHealthServices(zip, serviceType, false); // Call the fetchHealthServices function with the zip code
});

document.getElementById('locationBtn').addEventListener('click', async function(e) {
    e.preventDefault(); // Prevent form from reloading the page

    userLocation = await getUserLocation();
    const serviceType = document.getElementById('serviceType').value;
    fetchHealthServices(userLocation, serviceType, true); // Call the fetchHealthServices function with the zip code
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

function getUserLocation() {
    return new Promise((resolve, reject) => {
        if (navigator.geolocation) {
            navigator.geolocation.getCurrentPosition(
                (position) => {
                    const userLat = position.coords.latitude;
                    const userLng = position.coords.longitude;
                    resolve({ lat: userLat, lng: userLng });
                },
                (error) => {
                    console.error("Error getting user location:", error);
                    alert("Unable to retrieve your location. Please allow location access.");
                    reject(error);
                }
            );
        } else {
            alert("Geolocation is not supported by this browser.");
            reject(new Error("Geolocation not supported"));
        }
    });
}

function showUserLocation(map, userLocation) {
    map.setCenter(userLocation);

    // Add a marker for the user's location
    new google.maps.Marker({
        position: userLocation,
        map: map,
        title: "Your Location",
        icon: {
            url: "https://maps.google.com/mapfiles/ms/icons/blue-dot.png", // Blue marker for user
        },
    });

    console.log("User location:", userLocation);
}

loadGoogleMaps();
let currentInfoWindow = null;

// // Function to listen to clicks on the logout link element ID
// function setupLogoutListener() {
//     document.getElementById('logout-link').addEventListener('click', function(event) {
//         event.preventDefault();  // Prevent the default action (navigating to the href)
//         document.getElementById('logout-form').submit();  // Submit the form
//     });
// }

// setupLogoutListener();