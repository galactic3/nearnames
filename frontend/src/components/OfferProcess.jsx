import React from 'react'

function OfferProcessPage (props) {
  let finished = false;
  let success = false;
  if (props.connected) {
    finished = props.offerFinished;
    success = props.offerSuccess;
  }

  return (
    <div className='container my-auto'>
      {finished
        ? (success
          ? (
            <p className='alert alert-success'>
            Success!
            </p>
          ) : (
            <p className='alert alert-danger'>
            Something went wrong
            </p>
          )
        ) : (
          <div>
            <div className='d-flex justify-content-center' key='1'>
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
