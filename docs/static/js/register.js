window.onload = function() {
    // Check if the registration was successful (using a query parameter)
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.has('success') && urlParams.get('success') === 'true') {
        // Get the success alert element and display it
        const successAlert = document.getElementById('successAlert');
        if (successAlert) {
            successAlert.style.display = 'block';
        }
    }
};