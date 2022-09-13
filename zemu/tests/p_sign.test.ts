/** ******************************************************************************
 *  (c) 2020 Zondax GmbH
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
   You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ******************************************************************************* */

import Zemu from '@zondax/zemu'
import { cartesianProduct, curves, defaultOptions, models, ROOT_PATH } from './common'
import AvalancheApp, { Curve } from '@zondax/ledger-avalanche-app'
import {ADD_VALIDATOR_DATA, ADD_DELEGATOR_DATA, ADD_SUBNET_VALIDATOR_DATA, P_IMPORT_FROM_X, P_EXPORT_TO_X, CREATE_SUBNET, CREATE_CHAIN} from './p_chain_vectors'

// @ts-ignore
import secp256k1 from 'secp256k1/elliptic'
// @ts-ignore
import crypto from 'crypto'

const SIGN_TEST_DATA = cartesianProduct(curves, [
  {
    name: 'p_import_from_x',
    op: P_IMPORT_FROM_X ,
  },
  {
    name: 'p_export_to_x',
    op: P_EXPORT_TO_X ,
  },
  {
    name: 'add_validator',
    op: ADD_VALIDATOR_DATA,
  },
  {
    name: 'add_delegator',
    op: ADD_DELEGATOR_DATA,
  },
  {
    name: 'add_subnet_validator',
    op: ADD_SUBNET_VALIDATOR_DATA ,
  },
  {
    name: 'create_subnet',
    op: CREATE_SUBNET,
  },
  {
    name: 'create_chain',
    op: CREATE_CHAIN,
  },
])

describe.each(models)('P_Sign[%s]; sign', function (m) {
  test.each(SIGN_TEST_DATA)('sign p-chain transactions', async function (curve, data) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new AvalancheApp(sim.getTransport())
      const msg = data.op

      const testcase = `${m.prefix.toLowerCase()}-sign-${data.name}-${curve}`

      const signers = ["0/1", "5/8"];
      const respReq = app.sign(ROOT_PATH, signers, msg);

      await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())

      await sim.compareSnapshotsAndApprove('.', testcase)

      const resp = await respReq

      console.log(resp, m.name, data.name, curve)

      expect(resp.returnCode).toEqual(0x9000)
      expect(resp.errorMessage).toEqual('No errors')
      expect(resp).toHaveProperty('signatures')
      expect(resp.signatures?.size).toEqual(signers.length)

      switch (curve) {
        case Curve.Secp256K1:
          const hash = crypto.createHash('sha256')
          const msgHash = Uint8Array.from(hash.update(msg).digest())

          for (const signer of signers) {
            const path = `${ROOT_PATH}/${signer}`
            const resp_addr = await app.getAddressAndPubKey(path, false)
            const pk = Uint8Array.from(resp_addr.publicKey)
            const signatureRS = Uint8Array.from(resp.signatures?.get(signer)!).slice(1)

            const signatureOk = secp256k1.ecdsaVerify(signatureRS, msgHash, pk)
            expect(signatureOk).toEqual(true)
          }
          break

        default:
          throw Error('not a valid curve type')
      }
    } finally {
      await sim.close()
    }
  })
})

