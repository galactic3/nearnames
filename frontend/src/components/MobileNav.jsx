import React from 'react';
import {NavLink} from "react-router-dom";
import {IconButton} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import {nearToFloor, renderName} from "../utils";
import LogoutIcon from "@mui/icons-material/Logout";
import NetworkSelect from "./NetworkSelect";

function MobileNav(props) {

  console.log(props);


  return (
    <div className="mobile-header">
      <h1><NavLink aria-current='page' to='/' onClick={props.onClose}>Near names</NavLink></h1>
      <IconButton
        aria-label="close"
        onClick={props.onClose}
        className="button-icon"
        sx={{
          position: 'absolute',
          right: 20,
          top: 20,
          color: 'var(--gray)',
        }}
      >
        <CloseIcon />
      </IconButton>
      <NetworkSelect/>
      <ul className='nav'>
        <li className='nav-item'>
          <NavLink activeClassName='active' className='nav-link' aria-current='page'
                   onClick={props.onClose} to='/lots'>Lots</NavLink>
        </li>
        { props.signedAccount && (<li className='nav-item'>
          <NavLink activeClassName='active' className='nav-link' aria-current='page'
                   onClick={props.onClose} to='profile'>Profile</NavLink>
        </li>)}
        <li className='nav-item'>
          <NavLink activeClassName='active' className='nav-link' aria-current='page'
                   onClick={props.onClose} to='/about'>About</NavLink>
        </li>
      </ul>
      { !props.connected ? (
        <div className="auth">
          <span className='spinner' role='status' aria-hidden='true'>Connecting...</span>
        </div>
      ) : props.signedAccount
        ? <div className="auth">
          <strong className="balance near-icon">{nearToFloor(props.signedAccountBalance) || '-'}</strong>
          {renderName(props.signedAccount)}
          <a className="icon logout" onClick={() => props.signOut(true)}><LogoutIcon/></a>
        </div>
        : <div className="auth"><button className="login" onClick={props.signIn}>Log in</button></div>
      }
    </div>
  )
}

export default MobileNav;
