document.getElementById('zipForm').addEventListener('submit', async function(e) {
    e.preventDefault(); // Prevent form from reloading the page

    const zip = document.getElementById('zip').value; // Get the zip code from the input

    try {
        // Fetch data from the backend API
        const response = await fetch(`/services?zip=${zip}`);
        if (!response.ok) {
            throw new Error('Failed to fetch health services');
        }

        const data = await response.json(); // Parse the JSON response

        // Display the health services
        const resultsDiv = document.getElementById('results');
        resultsDiv.innerHTML = ''; // Clear previous results

        if (data.length === 0) {
            resultsDiv.innerHTML = '<p>No health services found.</p>';
        } else {
            const list = document.createElement('ul');
            data.forEach(service => {
                const listItem = document.createElement('li');
                listItem.textContent = `${service.name} - ${service.address}`;
                list.appendChild(listItem);
            });
            resultsDiv.appendChild(list);
        }
    } catch (error) {
        console.error('Error fetching health services:', error);
    }
});