import React from 'react'

export default function Loader () {
  return (
    <div className='d-flex m-5 justify-content-center' key='2'>
      <div className='spinner-grow' role='status'>
        <span className='visually-hidden'>Loading...</span>
      </div>
    </div>
  )
}