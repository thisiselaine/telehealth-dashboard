import React from 'react';

const DoctorCard = ({ doctor }) => {
  return (
    <div style={{ border: '1px solid #ddd', padding: '10px', margin: '10px 0' }}>
      <h3>{doctor.name}</h3>
      <p>{doctor.address}</p>
      <p>{doctor.practice}</p>
      <p>{doctor.timings}</p>
      <p>{doctor.distance} miles away</p>
      <img src={doctor.image} alt={`${doctor.name}`} style={{ width: '100px', height: '100px' }} />
    </div>
  );
};

export default DoctorCard;
