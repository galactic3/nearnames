import React from 'react'

function OfferProcessPage (props) {
  let finished = false;
  let success = false;
  if (props.connected) {
    finished = props.offerFinished;
    success = props.offerSuccess;
    offerFailureReason = props.offerFailureReason;
  }

  return (
    <div className='container'>
      {finished
        ? (success
          ? (
            <p className='alert alert-success'>
            Success!
            </p>
          ) : (
            <p className='alert alert-danger'>
              Something went wrong, prease refresh the page.
              <br/>
              {offerFailureReason && <span>Failure reason: {offerFailureReason}.</span>}
            </p>
          )
        ) : (
          <div>
            <div className='d-flex m-5 justify-content-center' key='1'>
              <div className='spinner-grow' role='status'>
                <span className='visually-hidden'>Loading...</span>
              </div>
            </div>
            <p className='alert alert-warning'>
            Do not refresh or close the page
            </p>

            <p className='alert alert-secondary'>
            It may take up to 5 minutes to complete
            </p>
          </div>
        )}

    </div>
  )
}

export default OfferProcessPage
