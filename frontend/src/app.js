import React, { useState } from 'react';
import Header from './components/Header';
import SearchFilters from './components/SearchFilters';
import Map from './components/Map';
import DoctorList from './components/DoctorList';
import './App.css';

function App() {
  const [searchResults, setSearchResults] = useState([]);

  const handleSearch = (results) => {
    setSearchResults(results);
  };

  return (
    <div className="App">
      <Header />
      <div className="search-container">
        <SearchFilters onSearch={handleSearch} />
        <Map />
      </div>
      <DoctorList doctors={searchResults} />
    </div>
  );
}

export default App;
