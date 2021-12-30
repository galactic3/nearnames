import React from 'react'

export default function Loader (props) {
  return (
    <div className={props.position === 'left' ? 'd-flex mx-5' : 'd-flex m-5 justify-content-center'} key={props.key || '1'}>
      <div className='spinner' role='status'>
        <span className='visually-hidden'>Loading...</span>
      </div>
    </div>
  )
}