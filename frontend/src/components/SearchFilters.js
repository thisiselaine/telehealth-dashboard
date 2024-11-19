import React, { useState } from 'react';
import axios from 'axios';

const SearchFilters = ({ onSearch }) => {
  const [zipCode, setZipCode] = useState('');
  const [specialty, setSpecialty] = useState('');

//   const handleSearch = async () => {
//     // Replace with your API endpoint when available
//     const response = await axios.post('API_ENDPOINT', {
//       zipCode,
//       specialty,
//     });
//     onSearch(response.data);
//   };

const handleSearch = () => {
    const mockData = [
      {
        name: 'Dr. Goober',
        address: '123 Health St.',
        practice: 'General Medicine',
        timings: '9 AM - 5 PM',
        distance: 1.2,
        image: 'https://via.placeholder.com/100',
      },
      // Add more mock doctors
    ];
    onSearch(mockData);
  };
  

  return (
    <div>
      <input
        type="text"
        placeholder="Enter ZIP code"
        value={zipCode}
        onChange={(e) => setZipCode(e.target.value)}
      />
      <select value={specialty} onChange={(e) => setSpecialty(e.target.value)}>
        <option value="">Select Specialty</option>
        <option value="cardiology">Cardiology</option>
        <option value="neurology">Neurology</option>
        <option value="orthopedics">Orthopedics</option>
        {/* Add more specialties */}
      </select>
      <button onClick={handleSearch}>Search</button>
    </div>
  );
};

export default SearchFilters;
