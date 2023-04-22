import keras.models as models
import keras.layers as layers
import keras.utils as utils
import keras.optimizers as optimizers

import keras.callbacks as callbacks

from keras.callbacks import ModelCheckpoint

import numpy

"""TENSOR FLOW BEGINS"""
def build_model_residual(conv_size, conv_depth):
  board3d = layers.Input(shape=(14, 8, 8))
  """
  magic from brian bong 
  model = Sequential()
  model.add(Dense(256, input_shape=(784,), activation="sigmoid"))
  model.add(Dense(128, activation="sigmoid"))
  model.add(Dense(10, activation="softmax"))
  """
  # adding the convolutional layers
  x = layers.Conv2D(filters=conv_size, kernel_size=3, padding='same')(board3d)
  for _ in range(conv_depth):
    previous = x
    x = layers.Conv2D(filters=conv_size, kernel_size=3, padding='same')(x)
    x = layers.BatchNormalization()(x)
    x = layers.Activation('relu')(x)
    x = layers.Conv2D(filters=conv_size, kernel_size=3, padding='same')(x)
    x = layers.BatchNormalization()(x)
    x = layers.Add()([x, previous])
    x = layers.Activation('relu')(x)
    x = layers.Flatten()(x)
    x = layers.Dense(1, 'sigmoid')(x)
  return models.Model(inputs=board3d, outputs=x)


"""TRAINING"""
def get_dataset():
	container = numpy.load('test.npz')
	board, eval1, eval2 = container['board_3d_arr'], container['eval']
	eval = numpy.asarray(eval / abs(eval).max() / 2 + 0.5, dtype=numpy.float32) # normalization (0 - 1)
	return board, eval



x_train, y_train = get_dataset()
#x_train.transpose()
#print(x_train.shape)
#print(y_train.shape)
#print(x_train[])
#print(y_train)


model = build_model_residual(32, 4)

model.compile(optimizer=optimizers.Adam(5e-4), loss='mean_squared_error')
model.summary()
checkpoint_filepath = '/tmp/checkpoint/'
model_checkpointing_callback = ModelCheckpoint(
    filepath = checkpoint_filepath,
    save_best_only= True,
)
model.fit(x_train, y_train,
          batch_size=2048,
          epochs=1000,
          verbose=1,
          validation_split=0.1,
          callbacks=[callbacks.ReduceLROnPlateau(monitor='loss', patience=10),
                     callbacks.EarlyStopping(monitor='loss', patience=15, min_delta=1e-4),model_checkpointing_callback])

model.save('model2.h5')


