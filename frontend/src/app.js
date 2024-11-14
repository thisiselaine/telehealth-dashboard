import React from 'react';
import { BrowserRouter as Router, Route, Switch } from 'react-router-dom';
import Header from './components/Header';
import Footer from './components/Footer';
import ProviderSearch from './components/ProviderSearch';
import AppointmentBooking from './components/AppointmentBooking';
import EducationalContent from './components/EducationalContent';
import Login from './components/Login';
import Register from './components/Register';
import Notifications from './components/Notifications';

function App() {
    return (
        <Router>
            <Header />
            <div className="container">
                <Switch>
                    <Route path="/" exact component={EducationalContent} />
                    <Route path="/providers" component={ProviderSearch} />
                    <Route path="/appointments" component={AppointmentBooking} />
                    <Route path="/login" component={Login} />
                    <Route path="/register" component={Register} />
                    <Route path="/notifications" component={Notifications} />
                </Switch>
            </div>
            <Footer />
        </Router>
    );
}

export default App;
