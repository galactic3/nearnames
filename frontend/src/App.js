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
import {nearTo, renderName} from "./utils";
import AboutPage from "./components/About";
import ConfirmContextProvider from "./Providers/ConfirmContextProvider";
import ModalConfirm from "./components/Confirm";

function App (props) {

  const lsPrevKeys = props.nearConfig.contractName + ':v01:' + 'prevKeys';
  const lsLotAccountId = props.nearConfig.contractName + ':v01:' + 'lotAccountId';

  const [connected, setConnected] = useState(false);
  const [signedAccount, setSignedAccount] = useState(props.currentUser && props.currentUser.accountId);
  const [signedAccountBalance, setSignedAccountBalance] = useState(props.currentUser && props.currentUser.balance);

  const [offerProcessState, setOfferProcessState] = useState({
    offerFinished: false,
    offerSuccess: false,
    offerFailureReason: '',
    offerSuccessMessage: '',
  })

  const [offerProcessOutput, setOfferProcessOutput] = useState([]);

  useEffect(async () => {
    await initOffer();
    setConnected(true);
  }, []);

  const updateBalance = async () => {
    setSignedAccountBalance(await getBalance(signedAccount));
  }

  const getBalance = async (accountId) => {
    debugger;
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
    setSignedAccount(props.currentUser && props.currentUser.accountId);
    withReload && window.location.replace(window.location.origin + window.location.pathname);
  };

  const initOffer = async() => {

    if (!signedAccount) {
      return;
    }

    const lotAccountId = localStorage.get(lsLotAccountId);
    if (!lotAccountId) {
      return;
    }

    if (signedAccount !== lotAccountId) {
      localStorage.remove(lsLotAccountId);
      const newState = {
        offerFinished: true,
        offerSuccess: false,
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

      const data = await with_timeout(fetch('/lock_unlock_account.wasm'));
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

      const offerResult = await with_timeout(props.contract.lot_offer(offerData));

      console.log(offerResult);

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
        offerSuccessMessage: `Account ${signedAccount} is now on sale. Log in as ${offerData.seller_id} to see it on your profile and be able collect rewards as soon as the first bid is made.`
      };
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...newState}));
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
        offerFailureReason
      };
      setOfferProcessState(offerProcessState => ({...offerProcessState, ...newState}));
    }
    signOut();
    console.log('init offer finish');
  }

  const passProps = {
    connected,
    signedAccount,
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
        <header>
          <div className="container">
            <h1><NavLink aria-current='page' to='/'>Near names</NavLink></h1>

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
                <CreateOffer {...{...passProps, ...offerProps}}/>
              { !connected ? (
                  <div className="auth">
                    <span className='spinner' role='status' aria-hidden='true'>Connecting...</span>
                  </div>
                ) : signedAccount
                ? <div className="auth">
                    <strong className="balance near-icon">{nearTo(signedAccountBalance) || '-'}</strong>
                    {renderName(signedAccount)}
                    <a className="icon logout" onClick={() => signOut(true)}><LogoutIcon/></a>
                  </div>
                : <div className="auth"><button className="login" onClick={signIn}>Log in</button></div>
              }
          </div>
        </header>
        <Switch>
          <Route exact path='/'>
            <Redirect to='/lots'/>
          </Route>
          <Route exact path='/lots'>
            <Lots {...passProps}/>
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
  </ConfirmContextProvider>
  )
}

export default App;

