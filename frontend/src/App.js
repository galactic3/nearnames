import 'regenerator-runtime/runtime';
import React from 'react';
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

class App extends React.Component {
  constructor(props) {
    super(props);

    this.app = {
      lsLotAccountId: props.nearConfig.contractName + ':v01:' + 'lotAccountId',
      lsPrevKeys: props.nearConfig.contractName + ':v01:' + 'prevKeys',
      near: props.near,
      wallet: props.wallet,
      config: props.nearConfig,
      contract: props.contract,
      currentUser: props.currentUser,
      account: props.wallet.account(),
      accountId: props.currentUser && props.currentUser.accountId,
    };

    this.state = {
      connected: false
    };

    this.app.updateBalance = async () => {
      this.setState({
        signedAccountBalance: await this.app.getBalance(this.app.accountId),
      })
    }

    this.getBalance = async (accountId) => {
      try {
        const account = await this.app.near.account(accountId);
        return nearTo((await account.getAccountBalance()).total);
      } catch (e) {
        return null;
      }
    }

    this.app.signIn = () => {
      props.wallet.requestSignIn(
        props.nearConfig.contractName,
        'Name hub'
      );
    };

    this.app.signOut = () => {
      props.wallet.signOut();
      window.location.replace(window.location.origin + window.location.pathname);
    };

    this.initApp().then(async () => {
      this.setState({
        signedIn: !!this.app.accountId,
        signedAccountId: this.app.accountId,
        signedAccountBalance: this.app.accountId && await this.getBalance(this.app.accountId),
        connected: true
      });
    })
  }

  async initApp () {

    /*this.app.logOut = () => {
      this.app.wallet.signOut();
      this.app.accountId = null;
      this.setState({
        signedIn: !!this.app.accountId,
        signedAccountId: this.app.accountId
      })
    };*/

    this.app.refreshAllowance = async () => {
      alert("You're out of access key allowance. Need sign in again to refresh it");
      await this.app.logOut();
      await this.signIn();
    };

    if (!this.app.accountId) {
      return;
    }

    const lotAccountId = localStorage.get(this.app.lsLotAccountId);
    if (!lotAccountId) {
      return;
    }

    if (this.app.accountId !== lotAccountId) {
      console.log(`wrong account, please try lot offer again`);
      localStorage.remove(this.app.lsLotAccountId);
      this.setState({
        offerFinished: true,
        offerSuccess: false,
        offerFailureReason: `wrong account authenticated, expected ${lotAccountId}, please try lot offer again`,
      })
      this.app.signOut();
      return;
    }

    // should never happen
    const offerData = JSON.parse(localStorage.get(this.app.config.contractName + ':lotOffer: ' + this.app.accountId));
    if (!offerData) {
      console.log(`failed to parse lot offer data`);
      localStorage.remove(this.app.lsLotAccountId);
      this.setState({
        offerFinished: true,
        offerSuccess: false,
        offerFailureReason: 'failed to parse lot offer data, please try lot offer again',
      })
      this.app.signOut();
      return;
    }

    const wrap_with_timeout = (promise, timeout_ms) => {
      const timer_promise =
        new Promise((resolve, reject) => setTimeout(() => reject("timeout_reached"), timeout_ms));
      return Promise.race([promise, timer_promise]);
    };
    const with_timeout = (promise) => wrap_with_timeout(promise, 60_000);

    try {
      const account = await with_timeout(this.app.near.account(this.app.accountId));
      console.log(lotAccountId);

      const lastKey = (await with_timeout(this.app.wallet._keyStore.getKey(this.app.config.networkId, this.app.accountId))).getPublicKey().toString();

      const accessKeys = await with_timeout(this.app.account.getAccessKeys());

      console.log('all keys', accessKeys);
      console.log('all local keys', this.app.wallet._authData.allKeys);
      console.log('last key', lastKey);

      const state = await with_timeout(account.state());
      console.log(state);

      const data = await with_timeout(fetch('/lock_unlock_account.wasm'));
      console.log('!', data);
      const buf = await with_timeout(data.arrayBuffer());

      await with_timeout(account.deployContract(new Uint8Array(buf)));

      const contractLock = await with_timeout(new nearAPI.Contract(account, this.app.accountId, {
        viewMethods: [],
        changeMethods: ['lock'],
        sender: this.app.accountId
      }));
      console.log('Deploying done. Initializing contract...');
      console.log(await with_timeout(contractLock.lock(Buffer.from('{"owner_id":"' + this.app.config.contractName + '"}'))));
      console.log('Init is done.');

      console.log('code hash', (await with_timeout(account.state())).code_hash);

      const offerResult = await with_timeout(this.app.contract.lot_offer(offerData));

      console.log(offerResult);

      for (let index = 0; index < accessKeys.length; index++) {
        if (accessKeys[index].public_key !== lastKey) {
          console.log('deleting ', accessKeys[index]);
          await with_timeout(account.deleteKey(accessKeys[index].public_key));
          console.log('deleting ', accessKeys[index], 'done');
        }
      }

      console.log('deleting last key', lastKey);
      await with_timeout(account.deleteKey(lastKey));
      console.log('deleting ', lastKey, 'done');

      localStorage.remove(this.app.config.contractName + ':lotOffer: ' + this.app.accountId);
      localStorage.remove(this.app.lsLotAccountId);
      this.setState({ offerFinished: true, offerSuccess: true })

      this.app.signOut()
    } catch (e) {
      console.log('Error', e)
      this.setState({ offerFinished: true, offerSuccess: false });
      e = e.toString();
      if (e === 'timeout_reached' || e === 'TypeError: NetworkError when attempting to fetch resource.') {
        this.setState({ offerFailureReason: "timeout on network operation reached, try reloading the page" })
      }
    }
    console.log('initapp finish');
  }

  render () {
    const passProps = {
      app: this.app,
      refreshAllowance: () => this.app.refreshAllowance(),
      ...this.state
    };

    return (
      <main>
        <Router basename='/'>
          <header>
            <div className="container">
              <h1><NavLink aria-current='page' to='/'>Near names</NavLink></h1>

              { this.state.connected &&
                <ul className='nav'>
                  <li className='nav-item'>
                    <NavLink activeClassName='active' className='nav-link' aria-current='page' to='/lots'>Lots</NavLink>
                  </li>
                { this.app.currentUser && (<li className='nav-item'>
                    <NavLink activeClassName='active' className='nav-link' aria-current='page'
                          to='profile'>Profile</NavLink>
                  </li>)}
                  <li className='nav-item'>
                    <NavLink activeClassName='active' className='nav-link' aria-current='page' to='/about'>About</NavLink>
                  </li>
                </ul> }

                <ConfirmContextProvider>
                  <CreateOffer {...passProps}/>
                  <ModalConfirm/>
                </ConfirmContextProvider>
                { !this.state.connected ? (
                    <div className="auth">
                      <span className='spinner' role='status' aria-hidden='true'>Connecting...</span>
                    </div>
                  ) : this.app.currentUser
                  ? <div className="auth">
                      <strong className="balance near-icon">{this.state.signedAccountBalance || '-'}</strong>
                      {renderName(this.app.accountId)}
                      <a className="icon logout" onClick={this.app.signOut}><LogoutIcon/></a>
                    </div>
                  : <div className="auth"><button className="login" onClick={this.app.signIn}>Log in</button></div>
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
              <OfferProcessPage {...passProps} />
            </Route>
            <Route exact path='/profile'>
              <ProfilePage {...passProps}/>
            </Route>
            <Route exact path='/about'>
              <AboutPage/>
            </Route>
          </Switch>
        </Router>
      </main>
    )
  }
}

export default App;

