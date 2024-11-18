import React from 'react';
import DoctorCard from './DoctorCard';

const DoctorList = ({ doctors }) => {
  return (
    <div className="doctor-list">
      {doctors.map((doctor, index) => (
        <DoctorCard key={index} doctor={doctor} />
      ))}
    </div>
  );
};

export default DoctorList;
