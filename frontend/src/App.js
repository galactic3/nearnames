import 'regenerator-runtime/runtime';
import React, {useEffect, useState} from 'react';
import * as nearAPI from 'near-api-js';
import localStorage from 'local-storage';
import {HashRouter as Router, NavLink, Redirect, Route, Switch} from 'react-router-dom';
import OfferProcessPage from './components/OfferProcess';
import Lots from './components/Lots';
import ProfilePage from './components/Profile';
import LogoutIcon from '@mui/icons-material/Logout';
import CreateOffer from "./components/CreateOffer";
import {CONFIG, nearToFloor, renderName} from "./utils";
import AboutPage from "./components/About";
import ConfirmContextProvider from "./Providers/ConfirmContextProvider";
import ModalConfirm from "./components/Confirm";
import {IconButton, MenuItem, Select} from "@mui/material";
import { BrowserView, MobileView, isBrowser, isMobile } from 'react-device-detect';
import MobileNav from "./components/MobileNav";
import MenuRoundedIcon from '@mui/icons-material/MenuRounded';
import NetworkSelect from "./components/NetworkSelect";

function App (props) {

  const lsPrevKeys = props.nearConfig.contractName + ':v01:' + 'prevKeys';
  const lsLotAccountId = props.nearConfig.contractName + ':v01:' + 'lotAccountId';

  const [connected, setConnected] = useState(false);
  const [showMobileNav, setShowMobileNav] = useState(false);
  const [signedAccount, setSignedAccount] = useState(props.currentUser && props.currentUser.accountId);
  const [signedAccountBalance, setSignedAccountBalance] = useState(props.currentUser && props.currentUser.balance);

  const [offerProcessState, setOfferProcessState] = useState({
    offerFinished: false,
    offerSuccess: false,
    offerActive: true,
    offerFailureReason: '',
    offerSuccessMessage: '',
  })

  const [offerProcessOutput, setOfferProcessOutput] = useState([]);

  useEffect(async () => {
    await initOffer();
    setConnected(true);

    console.log(process.env.NODE_ENV);
    console.log(process.env.NETWORK_ID);
    console.log(CONFIG);

  }, []);

  const updateBalance = async () => {
    setSignedAccountBalance(await getBalance(signedAccount));
  }

  const getBalance = async (accountId) => {
    try {
      const account = await props.near.account(accountId);
      return (await account.getAccountBalance()).total;
    } catch (e) {
      return null;
    }
  }

  const signIn = () => {
    props.wallet.requestSignIn(
      props.nearConfig.contractName,
      'Nearnames',
      window.location.origin + window.location.pathname
    );
  };

  const signOut = async (withReload) => {
    await props.wallet.signOut();
    setSignedAccount('');
    withReload && window.location.replace(window.location.origin + window.location.pathname);
  };

  const initOffer = async() => {

    if (!signedAccount) {
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...{offerActive: false}}));
      return;
    }

    const lotAccountId = localStorage.get(lsLotAccountId);
    if (!lotAccountId) {
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...{offerActive: false}}));
      return;
    }

    if (signedAccount !== lotAccountId) {
      localStorage.remove(lsLotAccountId);
      const newState = {
        offerFinished: true,
        offerSuccess: false,
        offerActive: true,
        offerFailureReason: `wrong account authenticated, expected ${lotAccountId}, please try lot offer again`,
      };
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...newState}));
      signOut();
      return;
    }

    // should never happen
    const offerData = JSON.parse(localStorage.get(props.nearConfig.contractName + ':lotOffer: ' + signedAccount));
    if (!offerData) {
      console.log(`failed to parse lot offer data`);
      localStorage.remove(lsLotAccountId);
      const newState = {
        offerFinished: true,
        offerSuccess: false,
        offerActive: true,
        offerFailureReason: 'failed to parse lot offer data, please try lot offer again',
      };
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...newState}));
      signOut();
      return;
    }

    const wrap_with_timeout = (promise, timeout_ms) => {
      const timer_promise =
        new Promise((resolve, reject) => setTimeout(() => reject("timeout_reached"), timeout_ms));
      return Promise.race([promise, timer_promise]);
    };
    const with_timeout = (promise) => wrap_with_timeout(promise, 60_000);

    try {

      const account = await with_timeout(props.near.account(signedAccount));

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'geting access keys']);

      const lastKey = (await with_timeout(props.wallet._keyStore.getKey(props.nearConfig.networkId, signedAccount))).getPublicKey().toString();

      const accessKeys = await with_timeout(props.wallet.account().getAccessKeys());

      console.log('all keys', accessKeys);
      console.log('all local keys', props.wallet._authData.allKeys);
      console.log('last key', lastKey);

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'fetching contract']);

      const data = await with_timeout(fetch('/lock_unlock_account_latest.wasm'));
      const buf = await with_timeout(data.arrayBuffer());

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'Deploying contract']);

      await with_timeout(account.deployContract(new Uint8Array(buf)));

      const contractLock = await with_timeout(new nearAPI.Contract(account, signedAccount, {
        viewMethods: [],
        changeMethods: ['lock'],
        sender: signedAccount
      }));

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'Deploying done. Initializing contract...']);
      console.log('Deploying done. Initializing contract...');
      console.log(await with_timeout(contractLock.lock(Buffer.from('{"owner_id":"' + props.nearConfig.contractName + '"}'))));

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'Init is done.']);
      console.log('Init is done.');

      console.log('code hash', (await with_timeout(account.state())).code_hash);

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'Create lot offer.']);

      const lot = await with_timeout(props.contract.lot_get({lot_id: lotAccountId}))

      if (!lot) {
        await with_timeout(props.contract.lot_offer(offerData));
      }

      for (let index = 0; index < accessKeys.length; index++) {
        if (accessKeys[index].public_key !== lastKey) {
          setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'deleting ' + accessKeys[index].public_key]);
          console.log('deleting ', accessKeys[index]);
          await with_timeout(account.deleteKey(accessKeys[index].public_key));
          console.log('deleting ', accessKeys[index], 'done');
        }
      }

      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'deleting last key ' + lastKey]);
      console.log('deleting last key', lastKey);
      await with_timeout(account.deleteKey(lastKey));
      setOfferProcessOutput(offerProcessOutput => [...offerProcessOutput, 'deleting done']);
      console.log('deleting ', lastKey, 'done');

      localStorage.remove(props.nearConfig.contractName + ':lotOffer: ' + signedAccount);
      localStorage.remove(lsLotAccountId);
      const newState = {
        offerFinished: true,
        offerSuccess: true,
        offerActive: true,
        offerSuccessMessage: `Account ${signedAccount} is now on sale. Log in as ${offerData.seller_id} to see it on your profile and be able collect rewards as soon as the first bid is made.`
      };
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...newState}));
      signOut();
    } catch (e) {
      console.log('Error', e)
      let offerFailureReason = '';
      e = e.toString();
      if (e === 'timeout_reached' || e === 'TypeError: NetworkError when attempting to fetch resource.') {
        offerFailureReason = 'timeout on network operation reached, try reloading the page';
      }
      const newState = {
        offerFinished: true,
        offerSuccess: false,
        offerActive: true,
        offerFailureReason
      };
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...newState}));
    } finally {
      console.log('init offer finish');
    }
  }

  const passProps = {
    connected,
    signedAccount,
    signedAccountBalance,
    ...props
  };

  const offerProps = {
    lsPrevKeys,
    lsLotAccountId,
  }

  return (
    <ConfirmContextProvider>
    <main>
      <Router basename='/'>
        <div className='beta-warning'>
          Beta software. Testnet version. Not audited. Use at your own risk!
        </div>
        <header>
          <div className="container">
            <h1><NavLink aria-current='page' to='/'>Near names</NavLink></h1>
            { isBrowser && <NetworkSelect/> }
            { !offerProcessState.offerActive && <BrowserView>
              <ul className='nav'>
                <li className='nav-item'>
                  <NavLink activeClassName='active' className='nav-link' aria-current='page' to='/lots'>Lots</NavLink>
                </li>
              { signedAccount && (<li className='nav-item'>
                  <NavLink activeClassName='active' className='nav-link' aria-current='page'
                        to='profile'>Profile</NavLink>
                </li>)}
                <li className='nav-item'>
                  <NavLink activeClassName='active' className='nav-link' aria-current='page' to='/about'>About</NavLink>
                </li>
              </ul>
            </BrowserView>}
            <CreateOffer {...{...passProps, ...offerProps}}/>
            { <BrowserView>
              { !connected ? (
                  <div className="auth">
                    <span className='spinner' role='status' aria-hidden='true'>Connecting...</span>
                  </div>
                ) : signedAccount && !offerProcessState.offerActive
                ? <div className="auth">
                    <strong className="balance near-icon">{nearToFloor(signedAccountBalance) || '-'}</strong>
                    {renderName(signedAccount)}
                    <a className="icon logout" onClick={() => signOut(true)}><LogoutIcon/></a>
                  </div>
                : <div className="auth"><button className="login" onClick={signIn}>Log in</button></div>
              }
            </BrowserView>}
            { !offerProcessState.offerActive && <MobileView>
              <IconButton
                aria-label="open"
                onClick={() => setShowMobileNav(true)}
                className="button-icon"
              >
                <MenuRoundedIcon />
              </IconButton>
              {showMobileNav && <MobileNav onClose={() => setShowMobileNav(false)} signIn={signIn} signOut={(e) => signOut(e)} {...passProps}/>}
            </MobileView> }
          </div>
        </header>
        <Switch>
          <Route exact path='/'>
            <Redirect to='/lots'/>
          </Route>
          <Route exact path='/lots'>
            <Lots {...{...passProps, signIn}}/>
          </Route>
          <Route exact path='/offerProcess'>
            <OfferProcessPage {...{...offerProcessState, offerProcessOutput}} />
          </Route>
          <Route exact path='/profile'>
            <ProfilePage {...{...passProps, updateBalance}}/>
          </Route>
          <Route exact path='/about'>
            <AboutPage/>
          </Route>
        </Switch>
      </Router>
      <ModalConfirm/>
    </main>
    <footer>
      <div className="container legal-notice">
        THE SERVICES ARE PROVIDED “AS IS” AND “AS AVAILABLE” WITHOUT WARRANTIES OF ANY
        KIND. WE DO NOT GUARANTEE, REPRESENT OR WARRANT THAT YOUR USE OF OUR SERVICE
        WILL BE UNINTERRUPTED, TIMELY, SECURE OR ERROR-FREE. WE DO NOT WARRANT THAT THE
        RESULTS THAT MAY BE OBTAINED FROM THE USE OF THE SERVICE WILL BE ACCURATE OR
        RELIABLE. YOU AGREE THAT FROM TIME TO TIME WE MAY REMOVE THE SERVICE FOR
        INDEFINITE PERIODS OF TIME OR CANCEL THE SERVICE AT ANY TIME, WITHOUT NOTICE TO
        YOU. YOU EXPRESSLY AGREE THAT YOUR USE OF, OR INABILITY TO USE, THE SERVICE IS
        AT YOUR SOLE RISK. THE SERVICE AND ALL PRODUCTS AND SERVICES DELIVERED TO YOU
        THROUGH THE SERVICE ARE (EXCEPT AS EXPRESSLY STATED BY US) PROVIDED 'AS IS' AND
        'AS AVAILABLE' FOR YOUR USE, WITHOUT ANY REPRESENTATION, WARRANTIES OR
        CONDITIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING ALL IMPLIED
        WARRANTIES OR CONDITIONS OF MERCHANTABILITY, MERCHANTABLE QUALITY, FITNESS FOR A
        PARTICULAR PURPOSE, DURABILITY, TITLE, AND NON-INFRINGEMENT. ALL INFORMATION
        SHOULD BE INDEPENDENTLY VERIFIED BEFORE MAKING A DECISION
      </div>
    </footer>
  </ConfirmContextProvider>
  )
}

export default App;

