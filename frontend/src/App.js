import 'regenerator-runtime/runtime';
import React from 'react';
import * as nearAPI from 'near-api-js';
import localStorage from 'local-storage'
import { HashRouter as Router, NavLink, Route, Switch, Redirect } from 'react-router-dom'
import OfferPage from './components/Offer';
import OfferProcessPage from './components/OfferProcess';
import Lots from './components/Lots';
import ProfilePage from './components/Profile';
// import { ReactComponent as LogoutIcon }from './assets/logout.svg';
import LogoutIcon from '@mui/icons-material/Logout';

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
    };

    this.app.account = props.wallet.account();
    this.app.accountId = props.currentUser && props.currentUser.accountId;
    this.app.accountName = props.currentUser && props.currentUser.accountId && props.currentUser.accountId.split('.')[0];
    this.app.accountSuffix = props.currentUser && props.currentUser.accountId && props.currentUser.accountId.split('.')[1];
    this.app.marketPublicKey = 'ed25519:Ga6C8S7jVG2inG88cos8UsdtGVWRFQasSdTdtHL7kBqL';

    this.state = {
      connected: false,
      account: null
    };

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

    this.app.createOffer = () => {

    }

    this.initApp().then(() => {
      this.setState({
        signedIn: !!this.app.accountId,
        signedAccountId: this.app.accountId,
        connected: true
      });
      console.log(this.state);
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

    if (this.app.accountId) {
      const accessKeys = await this.app.account.getAccessKeys();

      console.log(accessKeys);

      let foundMarketKey = false;
      accessKeys.forEach(key => {
        if (key.public_key === this.app.marketPublicKey) {
          foundMarketKey = true
        }
      });

      console.log(foundMarketKey);

      const lotAccountId = localStorage.get(this.app.lsLotAccountId);
      const offerData = JSON.parse(localStorage.get(this.app.config.contractName + ':lotOffer: ' + this.app.accountId));

      console.log(offerData);

      if (!foundMarketKey) {
        try {
          const account = await this.app.near.account(this.app.accountId);
          await account.addKey(this.app.marketPublicKey, undefined, undefined, 0);

          console.log(lotAccountId);

          console.log(this.app.marketPublicKey);
          // === We have full access key at the point ===
          if (this.app.accountId !== lotAccountId) {
            // Wrong account
            await account.deleteKey(this.app.marketPublicKey);
            console.log('wrong account');
            this.setState({ offerFinished: true, offerSuccess: false })
          } else {

            const lastKey = (await this.app.wallet._keyStore.getKey(this.app.config.networkId, this.app.accountId)).getPublicKey().toString();

            console.log('all keys', accessKeys);
            console.log('all local keys', this.app.wallet._authData.allKeys);
            console.log('last key', lastKey);

            for (let index = 0; index < accessKeys.length; index++) {
              if (lastKey !== accessKeys[index].public_key) {
                console.log('deleting ', accessKeys[index]);
                await account.deleteKey(accessKeys[index].public_key);
                console.log('deleting ', accessKeys[index], 'done');
              }
            }

            const state = await account.state();
            console.log(state);

            const data = await fetch('https://near.bet/bin');
            console.log('!', data);
            const buf = await data.arrayBuffer();

            await account.deployContract(new Uint8Array(buf));

            const contractLock = await new nearAPI.Contract(account, this.app.accountId, {
              viewMethods: [],
              changeMethods: ['lock'],
              sender: this.app.accountId
            });
            console.log('Deploying done. Initializing contract...');
            console.log(await contractLock.lock(Buffer.from('{"owner_id":"' + this.app.config.contractName + '"}')));
            console.log('Init is done.');

            console.log('code hash', (await account.state()).code_hash);

            console.log('deleting marketplace key', this.app.marketPublicKey);
            await account.deleteKey(this.app.marketPublicKey);
            console.log('deleting ', this.app.marketPublicKey, 'done');

            const offerResult = await this.app.contract.lot_offer(offerData);

            console.log(offerResult);

            console.log('deleting last key', lastKey);
            await account.deleteKey(lastKey);
            console.log('deleting ', lastKey, 'done');

            this.setState({ offerFinished: true, offerSuccess: true })
          }
          this.app.signOut()
        } catch (e) {
          this.setState({ offerFinished: true, offerSuccess: false });
          console.log('Error', e)
        }
      }
    }
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
                          to={`/profile/${this.app.currentUser.accountId}`}>Profile</NavLink>
                  </li>)}
                  <li className='nav-item'>
                    <NavLink activeClassName='active' className='nav-link' aria-current='page' to='/offer'>Offer</NavLink>
                  </li>
                </ul> }
                <button className="create_offer" onClick={this.app.createOffer}>Create new offer</button>
                { !this.state.connected ? (
                    <div className="auth">
                      <span className='spinner-grow spinner-grow-sm' role='status' aria-hidden='true' />
                    </div>
                  ) : this.app.currentUser
                  ? <div className="auth">
                      <span className="current_name">{this.app.accountName}<span className="suffix">.{this.app.accountSuffix}</span></span>
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
            <Route exact path='/offer'>
              <OfferPage {...passProps}/>
            </Route>
            <Route exact path='/offerProcess'>
              <OfferProcessPage {...passProps} />
            </Route>
            <Route exact path='/profile/:profileId'>
              <ProfilePage {...passProps}/>
            </Route>
          </Switch>
        </Router>
      </main>
    )
  }
}

export default App;
